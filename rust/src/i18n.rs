//lazy_static! {
//    static ref CATALOGS = include_i18n!()
//}

use gettext::Catalog;
use std::fs::File;

init_i18n!("pixelflut", po = false, en);
compile_i18n!();

lazy_static! {
    static ref CATALOGS: Vec<(&'static str, Catalog)> = include_i18n!();
    static ref DUMMY: Catalog = Catalog::empty();
    static ref ENGLISH: &'static Catalog = find_catalog("en");
}

fn find_catalog(language: &str) -> &Catalog {
    let catalogs = &CATALOGS;
    catalogs
        .iter()
        .find(|(i_lang, catalog)| *i_lang == language)
        .map(|(i_lang, catalog)| catalog)
        .expect(&format!("Could not find i18n catalog for {}", language))
}

pub(crate) fn get_catalog() -> &'static Catalog {
    &ENGLISH
    //&DUMMY
}
