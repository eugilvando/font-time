//! Lee de un .ttf/.otf/.ttc solo las tablas que hacen falta (name, OS/2 y,
//! para el mapa de caracteres, cmap). La indexación también calcula una huella del contenido y métricas por cara.

use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use super::categoria::{self, Datos};

/// Más que esto en una tabla name/OS/2/cmap es un archivo roto o malicioso.
const TOPE_TABLA: u32 = 32 * 1024 * 1024;
const TOPE_CARAS: u32 = 64;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Propiedades {
    pub variable: bool,
    pub color: bool,
    pub contraste: Option<u16>,
    pub peso: Option<u16>,
    pub proporcion: Option<u16>,
    pub altura: Option<u16>,
    pub huella: String,
}

fn propiedades(face: Option<ttf_parser::Face>, panose: &[u8;10], huella: &str) -> Propiedades {
    let contraste = if panose[0]==2 && (2..=9).contains(&panose[4]) { Some((u16::from(panose[4])-2)*100/7) } else {None};
    let mut p=Propiedades {contraste,huella:huella.into(),..Default::default()};
    if let Some(f)=face {
        p.peso=f.tables().os2.map(|_|f.weight().to_number());
        let em=u32::from(f.units_per_em());
        let percent=|v:u16| (u32::from(v)*100/em) as u16;
        p.proporcion=f.glyph_index('n').and_then(|id|f.glyph_hor_advance(id)).map(percent);
        p.altura=f.x_height().or_else(|| f.glyph_index('x').and_then(|id|f.glyph_bounding_box(id)).map(|r|r.y_max))
            .filter(|h|*h>0).map(|h|percent(h as u16));
        p.variable=f.is_variable();
        p.color=[b"COLR",b"CBDT",b"sbix",b"SVG "].iter().any(|tag|f.raw_face().table(ttf_parser::Tag::from_bytes(tag)).is_some());
    }
    p
}

/// Una cara (estilo) dentro de un archivo.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Cara {
    /// Familia tipográfica (name 16, o 1 si no hay).
    pub familia: String,
    /// Familia "de siempre" (name 1): la que entiende Windows al pedirla por nombre.
    pub familia_win: String,
    /// Estilo (name 17, o 2).
    pub estilo: String,
    pub completo: String,
    #[serde(default)]
    pub postscript: String,
    pub peso: u16,
    pub cursiva: bool,
    pub categoria: String,
    #[serde(default)]
    pub propiedades: Propiedades,
}

fn be16(b: &[u8], i: usize) -> Option<u16> {
    b.get(i..i + 2).map(|s| u16::from_be_bytes([s[0], s[1]]))
}
fn be32(b: &[u8], i: usize) -> Option<u32> {
    b.get(i..i + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

fn leer_en(f: &mut File, desde: u64, largo: usize) -> std::io::Result<Vec<u8>> {
    let mut v = vec![0; largo];
    f.seek(SeekFrom::Start(desde))?;
    f.read_exact(&mut v)?;
    Ok(v)
}

/// Las tablas pedidas (por etiqueta) de la fuente que empieza en `inicio`.
fn tablas(f: &mut File, inicio: u64, quiero: &[&[u8; 4]]) -> Result<Vec<Option<Vec<u8>>>, String> {
    let cab = leer_en(f, inicio, 12).map_err(|e| e.to_string())?;
    let n = be16(&cab, 4).ok_or("cabecera corta")? as usize;
    if n == 0 || n > 512 {
        return Err("directorio de tablas raro".into());
    }
    let dir = leer_en(f, inicio + 12, n * 16).map_err(|e| e.to_string())?;
    let mut fuera = vec![None; quiero.len()];
    for r in dir.chunks_exact(16) {
        let Some(i) = quiero.iter().position(|t| &r[0..4] == *t) else { continue };
        let (off, len) = (be32(r, 8).unwrap_or(0), be32(r, 12).unwrap_or(0));
        if len == 0 || len > TOPE_TABLA {
            continue;
        }
        fuera[i] = Some(leer_en(f, off as u64, len as usize).map_err(|e| e.to_string())?);
    }
    Ok(fuera)
}

/// Desplazamientos de cada fuente del archivo (una, o varias si es .ttc).
fn inicios(f: &mut File) -> Result<Vec<u64>, String> {
    let cab = leer_en(f, 0, 12).map_err(|_| "no es una fuente".to_string())?;
    match &cab[0..4] {
        b"ttcf" => {
            let n = be32(&cab, 8).unwrap_or(0).min(TOPE_CARAS) as usize;
            let offs = leer_en(f, 12, n * 4).map_err(|e| e.to_string())?;
            Ok(offs.chunks_exact(4).map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]) as u64).collect())
        }
        [0, 1, 0, 0] | b"OTTO" | b"true" => Ok(vec![0]),
        _ => Err("no es una fuente".into()),
    }
}

/// Texto de un nombre: primero Windows en inglés, luego cualquier Windows, luego lo que haya.
fn nombre(t: &ttf_parser::name::Table, id: u16) -> Option<String> {
    let mut mejor: Option<(u8, String)> = None;
    for n in t.names {
        if n.name_id != id {
            continue;
        }
        let Some(s) = n.to_string() else { continue };
        let s = s.trim().to_string();
        if s.is_empty() {
            continue;
        }
        let nota = match (n.platform_id, n.language_id) {
            (ttf_parser::PlatformId::Windows, 0x409) => 3,
            (ttf_parser::PlatformId::Windows, _) => 2,
            _ => 1,
        };
        if mejor.as_ref().is_none_or(|(m, _)| nota > *m) {
            mejor = Some((nota, s));
        }
    }
    mejor.map(|(_, s)| s)
}

fn sin_extension(ruta: &Path) -> String {
    ruta.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Ejes declarados por la cara de la familia solicitada.
pub fn ejes(ruta: &Path, familia: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut file = File::open(ruta).map_err(|e| e.to_string())?;
    for inicio in inicios(&mut file)? {
        let t = tablas(&mut file, inicio, &[b"name", b"fvar"])?;
        let Some(names) = t[0].as_deref().and_then(ttf_parser::name::Table::parse) else {continue};
        if nombre(&names, 16).or_else(||nombre(&names,1)).as_deref()!=Some(familia) {continue;}
        let Some(table) = t[1].as_deref().and_then(ttf_parser::fvar::Table::parse) else {continue};
        return Ok(table.axes.into_iter().filter(|axis| !axis.hidden).map(|axis| serde_json::json!({
            "tag": axis.tag.to_string(), "label": nombre(&names,axis.name_id).unwrap_or_else(||axis.tag.to_string()),
            "min": axis.min_value, "max": axis.max_value, "value": axis.def_value
        })).collect());
    }
    Ok(Vec::new())
}

/// Todas las caras de un archivo, con su categoría.
pub fn caras(ruta: &Path) -> Result<Vec<Cara>, String> {
    let mut f = File::open(ruta).map_err(|e| e.to_string())?;
    let bytes=std::fs::read(ruta).map_err(crate::err)?;
    use sha2::Digest;
    let huella=format!("{:x}",sha2::Sha256::digest(&bytes));
    let mut fuera = Vec::new();
    for (index,inicio) in inicios(&mut f)?.into_iter().enumerate() {
        let t = tablas(&mut f, inicio, &[b"name", b"OS/2"])?;
        let name = t[0].as_deref().and_then(ttf_parser::name::Table::parse).ok_or("sin tabla name")?;
        let os2 = t[1].as_deref();

        let familia_win = nombre(&name, 1).unwrap_or_else(|| sin_extension(ruta));
        let familia = nombre(&name, 16).unwrap_or_else(|| familia_win.clone());
        let estilo = nombre(&name, 17).or_else(|| nombre(&name, 2)).unwrap_or_else(|| "Regular".into());
        let completo = nombre(&name, 4).unwrap_or_else(|| format!("{familia} {estilo}"));

        let mut d = Datos { familia: familia.clone(), ..Default::default() };
        let (mut peso, mut cursiva) = (400, estilo.to_lowercase().contains("italic"));
        if let Some(o) = os2 {
            peso = be16(o, 4).filter(|p| (1..=1000).contains(p)).unwrap_or(400);
            let sel = be16(o, 62).unwrap_or(0);
            cursiva = sel & 1 != 0 || sel & (1 << 9) != 0;
            d.clase = be16(o, 30).unwrap_or(0) as i16;
            if let Some(p) = o.get(32..42) {
                d.panose.copy_from_slice(p);
            }
            for (i, r) in d.unicode.iter_mut().enumerate() {
                *r = be32(o, 42 + i * 4).unwrap_or(0);
            }
            if be16(o, 0).unwrap_or(0) >= 1 {
                d.codepages = be32(o, 78);
            }
        }
        let props=propiedades(ttf_parser::Face::parse(&bytes,index as u32).ok(),&d.panose,&huella);
        fuera.push(Cara { propiedades:props, familia, familia_win, estilo, completo, postscript: nombre(&name,6).unwrap_or_default(), peso, cursiva, categoria: categoria::clasificar(&d).into() });
    }
    if fuera.is_empty() {
        return Err("el archivo no trae ninguna fuente".into());
    }
    Ok(fuera)
}

/// Caracteres que tiene de verdad la primera fuente del archivo (tabla cmap),
/// sin controles ni espacios, como mucho `tope`.
pub fn caracteres(ruta: &Path, tope: usize) -> Result<Vec<u32>, String> {
    let mut f = File::open(ruta).map_err(|e| e.to_string())?;
    let inicio = *inicios(&mut f)?.first().ok_or("vacío")?;
    let t = tablas(&mut f, inicio, &[b"cmap"])?;
    let cmap = t[0].as_deref().and_then(ttf_parser::cmap::Table::parse).ok_or("sin tabla cmap")?;
    let mut set = std::collections::BTreeSet::new();
    for sub in cmap.subtables {
        if !sub.is_unicode() {
            continue;
        }
        sub.codepoints(|c| {
            if let Some(ch) = char::from_u32(c) {
                if !ch.is_control() && !ch.is_whitespace() && !(0xE000..=0xF8FF).contains(&c) {
                    set.insert(c);
                }
            }
        });
    }
    // Fuentes de símbolos (Wingdings y compañía): solo cmap de símbolo, en la zona privada.
    if set.is_empty() {
        for sub in cmap.subtables {
            sub.codepoints(|c| {
                if (0xF020..=0xF0FF).contains(&c) {
                    set.insert(c);
                }
            });
        }
    }
    Ok(set.into_iter().take(tope).collect())
}
