use super::*;

#[test]
fn importar_sin_sobrescribir_ni_duplicar() {
    let tmp=tempfile::tempdir().unwrap();
    let dir=tmp.path().join("OneDrive");
    fs::create_dir(&dir).unwrap();
    let fuente=tmp.path().join("font.ttf");
    fs::write(&fuente,b"nuevo").unwrap();
    fs::write(dir.join("font.ttf"),b"original").unwrap();
    copiar_importacion(&fuente,&dir).unwrap();
    copiar_importacion(&fuente,&dir).unwrap();
    assert_eq!(fs::read(dir.join("font (2).ttf")).unwrap(),b"nuevo");
    assert_eq!(fs::read(dir.join("font.ttf")).unwrap(),b"original");
    assert_eq!(fs::read_dir(&dir).unwrap().count(),2);
    let sub=dir.join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("dentro.ttf"),b"dentro").unwrap();
    copiar_importacion(&sub.join("dentro.ttf"),&dir).unwrap();
    assert!(!dir.join("dentro.ttf").exists());
    assert!(copiar_importacion(&fuente,&tmp.path().join("offline")).is_err());
}

#[test]
fn identicos_y_metricas_por_estilo() {
    let tmp=tempfile::tempdir().unwrap();
    let fuente=Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/GochiHand-Regular.ttf"));
    let entradas:Vec<_>=["uno.ttf","dos.ttf"].iter().map(|nombre| {
        let ruta=tmp.path().join(nombre);
        fs::copy(fuente,&ruta).unwrap();
        fuentes::Entrada{ruta,origen:Origen::Carpeta,activa:false,valor:None}
    }).collect();
    let mut cache=Cache::default();
    assert_eq!(cache.actualizar(&entradas),2);
    let s=&cache.familias[0].estilos[0];
    assert!(s.duplicada && s.identicos.len()==2 && s.propiedades.peso==Some(400));
    assert!(s.propiedades.altura.is_some() && s.propiedades.proporcion.is_some());
    assert_eq!(cache.actualizar(&entradas),0);
    cache.actualizar(&entradas[..1]);
    assert!(!cache.familias[0].estilos[0].duplicada);
}

#[cfg(windows)]
#[test]
fn alternar_negrita_no_cambia_regular() {
    let tmp=tempfile::tempdir().unwrap();
    let base=format!("Software\\FontTimeTest-{}",std::process::id());
    let ent=Entorno { fuentes_sistema:tmp.path().join("system"),fuentes_usuario:tmp.path().join("user"),
        clave_sistema:format!("{base}\\System"),clave_usuario:format!("{base}\\User"),datos:tmp.path().join("data"),
        tocar_windows:false,de_windows:|_| false,de_programas:HashSet::new() };
    let dir=ent.biblioteca();
    fs::create_dir_all(&dir).unwrap();
    let sistema=PathBuf::from(std::env::var_os("WINDIR").unwrap()).join("Fonts");
    for n in ["arial.ttf","arialbd.ttf"] {fs::copy(sistema.join(n),dir.join(n)).unwrap();}
    let mut cache=Cache::default();
    let mut entradas=fuentes::entradas(&ent,&[]);
    cache.actualizar(&entradas);
    let familia=cache.familias.iter().find(|f| f.nombre=="Arial").unwrap();
    let bold=familia.estilos.iter().find(|s| s.peso==700).unwrap();
    let id=SeleccionEstilo{familia:familia.nombre.clone(),nombre:bold.nombre.clone(),peso:bold.peso,cursiva:bold.cursiva};
    for activar in [true,false] {
        let mut r=cambios::Resultado::default();
        alternar_estilos(&ent,&entradas,&cache,std::slice::from_ref(&id),activar,&mut r);
        assert_eq!(r.cambiadas,1,"{:?}",r.errores);
        entradas=fuentes::entradas(&ent,&[]);
        cache.actualizar(&entradas);
        let f=cache.familias.iter().find(|f|f.nombre=="Arial").unwrap();
        assert!(f.estilos.iter().all(|s|s.activa==(activar && s.peso==700)));
        assert_eq!(fs::read_dir(&dir).unwrap().count(),2);
    }
    let _=winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).delete_subkey_all(base);
}
