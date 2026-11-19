use crate::{err, fondo, Estado};
use serde::Serialize;
use std::{collections::BTreeMap, path::{Path, PathBuf}};
use tauri::{AppHandle, Manager};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

#[derive(serde::Deserialize)]
pub struct Fuente { archivo: PathBuf, nombre: String, peso: u16, cursiva: bool, familia: String }

fn indice(app: &AppHandle, fuente: &Fuente) -> Result<u32,String> {
    let estado = app.state::<Estado>();
    let cache = estado.cache.lock().unwrap();
    let caras = cache.caras(&fuente.archivo).ok_or("Font is no longer in the library")?;
    caras.iter().position(|c| c.familia.eq_ignore_ascii_case(&fuente.familia) && c.estilo == fuente.nombre && c.peso == fuente.peso && c.cursiva == fuente.cursiva)
        .map(|i| i as u32).ok_or_else(|| "Style unavailable".into())
}

#[derive(Serialize)]
pub struct Mapa { nombres: Vec<String>, codigos: Vec<(u32,u16)> }

fn mapa(path: &Path, index: u32) -> Result<Mapa,String> {
    let bytes = std::fs::read(path).map_err(err)?;
    let face = Face::parse(&bytes,index).map_err(err)?;
    let nombres = (0..face.number_of_glyphs()).map(|i| face.glyph_name(GlyphId(i)).unwrap_or("").to_owned()).collect();
    let mut codigos = BTreeMap::new();
    if let Some(cmap) = face.tables().cmap {
        for sub in cmap.subtables {
            if !(sub.is_unicode() || (sub.platform_id == ttf_parser::PlatformId::Windows && sub.encoding_id == 0)) { continue; }
            sub.codepoints(|cp| {
                if char::from_u32(cp).is_some() {
                    if let Some(g) = sub.glyph_index(cp).filter(|g| g.0 > 0 && g.0 < face.number_of_glyphs()) {
                        codigos.entry(cp).or_insert(g.0);
                    }
                }
            });
        }
    }
    Ok(Mapa { nombres, codigos:codigos.into_iter().collect() })
}

#[tauri::command]
pub async fn mapa_glifos(app:AppHandle, fuente:Fuente) -> Result<Mapa,String> {
    fondo(move || mapa(&fuente.archivo, indice(&app,&fuente)?)).await
}

#[tauri::command]
pub async fn licencia_fuente(app:AppHandle, fuente:Fuente) -> Result<serde_json::Value,String> {
    fondo(move || {
        let i=indice(&app,&fuente)?;
        let bytes=std::fs::read(&fuente.archivo).map_err(err)?;
        let f=Face::parse(&bytes,i).map_err(err)?;
        let texto=|id| f.names().into_iter().filter(|n|n.name_id==id).filter_map(|n|Some((n.language_id==0x409,n.to_string()?))).max_by_key(|(ingles,_)|*ingles).map(|(_,s)|s).unwrap_or_default();
        Ok(serde_json::json!({"terms":texto(13),"url":texto(14),"copyright":texto(0)}))
    }).await
}

struct Trazo(String);
impl OutlineBuilder for Trazo {
    fn move_to(&mut self,x:f32,y:f32){self.0.push_str(&format!("M{x} {}",-y));}
    fn line_to(&mut self,x:f32,y:f32){self.0.push_str(&format!("L{x} {}",-y));}
    fn quad_to(&mut self,x1:f32,y1:f32,x:f32,y:f32){self.0.push_str(&format!("Q{x1} {} {x} {}",-y1,-y));}
    fn curve_to(&mut self,x1:f32,y1:f32,x2:f32,y2:f32,x:f32,y:f32){self.0.push_str(&format!("C{x1} {} {x2} {} {x} {}",-y1,-y2,-y));}
    fn close(&mut self){self.0.push('Z');}
}

#[tauri::command]
pub async fn trazos_glifos(app:AppHandle, fuente:Fuente, ids:Vec<u16>) -> Result<BTreeMap<u16,(String,String)>,String> {
    if ids.len() > 128 { return Err("Page too large".into()); }
    fondo(move || {
        let i=indice(&app,&fuente)?;
        let bytes=std::fs::read(&fuente.archivo).map_err(err)?;
        let face=Face::parse(&bytes,i).map_err(err)?;
        let mut paths=BTreeMap::new();
        for id in ids {
            let mut p=Trazo(String::new());
            if let Some(r)=face.outline_glyph(GlyphId(id),&mut p) {
                let em=i32::from(face.units_per_em());
                let width=(i32::from(r.x_max)-i32::from(r.x_min)).max(em/2);
                let top=i32::from(face.ascender()).max(i32::from(r.y_max));
                let bottom=i32::from(face.descender()).min(i32::from(r.y_min));
                paths.insert(id,(format!("{} {} {} {}",r.x_min,-top,width,(top-bottom).max(1)),p.0));
            }
        }
        Ok(paths)
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mapa_completo() {
        let m=mapa(Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/GochiHand-Regular.ttf")),0).unwrap();
        assert!(!m.nombres.is_empty());
        assert!(m.codigos.iter().any(|(c,_)| *c==65));
        assert!(m.codigos.iter().all(|(_,g)| usize::from(*g)<m.nombres.len()));
    }
}
