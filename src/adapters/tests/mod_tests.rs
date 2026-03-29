use super::for_kind;
use crate::recipe::schema::RecipeKind;

#[test]
fn for_kind_brew_returns_brew_adapter() {
    // arrange / act
    let adapter = for_kind(&RecipeKind::Brew).unwrap();
    // assert
    assert_eq!(adapter.name(), "brew");
}

#[test]
fn for_kind_apt_returns_apt_adapter() {
    // arrange / act
    let adapter = for_kind(&RecipeKind::Apt).unwrap();
    // assert
    assert_eq!(adapter.name(), "apt");
}

#[test]
fn for_kind_pacman_returns_pacman_adapter() {
    // arrange / act
    let adapter = for_kind(&RecipeKind::Pacman).unwrap();
    // assert
    assert_eq!(adapter.name(), "pacman");
}

#[test]
fn for_kind_qwert_returns_none() {
    // arrange / act / assert
    assert!(for_kind(&RecipeKind::Qwert).is_none());
}
