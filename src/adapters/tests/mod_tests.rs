use super::for_kind;
use crate::recipe::schema::RecipeKind;

#[test]
fn for_kind_brew_returns_brew_adapter() {
    // arrange
    let kind = RecipeKind::Brew;
    // act
    let adapter = for_kind(&kind).unwrap();
    // assert
    assert_eq!(adapter.install_cmd("x"), "brew install x");
}

#[test]
fn for_kind_apt_returns_apt_adapter() {
    // arrange
    let kind = RecipeKind::Apt;
    // act
    let adapter = for_kind(&kind).unwrap();
    // assert
    assert_eq!(adapter.install_cmd("x"), "sudo apt-get install -y x");
}

#[test]
fn for_kind_pacman_returns_pacman_adapter() {
    // arrange
    let kind = RecipeKind::Pacman;
    // act
    let adapter = for_kind(&kind).unwrap();
    // assert
    assert_eq!(adapter.install_cmd("x"), "sudo pacman -S --noconfirm x");
}

#[test]
fn for_kind_qwert_returns_none() {
    // arrange
    let kind = RecipeKind::Qwert;
    // act
    let result = for_kind(&kind);
    // assert
    assert!(result.is_none());
}
