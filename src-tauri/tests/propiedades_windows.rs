#![cfg(windows)]
use std::path::PathBuf;
use tunafonts_lib::fuentes::leer;

#[test]
fn fuentes_de_referencia_de_windows() {
    let dir=PathBuf::from(std::env::var_os("WINDIR").unwrap()).join("Fonts");
    for (file,variable,color) in [("SegUIVar.ttf",true,false),("seguiemj.ttf",false,true)] {
        let path=dir.join(file);
        if path.is_file() {
            let caras=leer::caras(&path).unwrap();
            assert!(caras.iter().any(|c|c.propiedades.variable==variable && c.propiedades.color==color),"{file}");
            println!("{file}: {} faces verified",caras.len());
        }
    }
    let coleccion=dir.join("msyh.ttc");
    if coleccion.is_file() {
        let caras=leer::caras(&coleccion).unwrap();
        assert!(caras.len()>1);
        let caracteres=leer::caracteres(&coleccion,usize::MAX).unwrap();
        assert!(caracteres.len()>4000);
        println!("msyh.ttc: {} faces, {} mapped characters",caras.len(),caracteres.len());
    }
}
