// Test minimal pour valider la cible et l'affichage des logs

#[test]
fn poc_sign() {
    println!("[LOG] Test factice Monero Oxide lancé !");
    let a = 42;
    let b = 24;
    println!("[LOG] Valeur a = {}", a);
    println!("[LOG] Valeur b = {}", b);
    println!("[LOG] Somme simulée = {}", a + b);
    assert_eq!(a + b, 66);
}

// Remarque : Les champs doivent être adaptés selon la structure réelle si Default n'est pas disponible.
// Ce script doit être ajusté si les types ou signatures diffèrent.
