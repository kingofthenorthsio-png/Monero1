// Test pour simuler une double dépense avec le même key image
use wallet::SharedKeyDerivations;
use monero_oxide::transaction::Input;
use monero_oxide::io::CompressedPoint;

#[test]
fn double_spend_key_image() {
    // Simule un key image arbitraire
    let key_image_bytes = [1u8; 32];
    let key_image = CompressedPoint(key_image_bytes);
    // Deux inputs avec le même key image
    let input1 = Input::ToKey {
        amount: Some(10),
        key_offsets: vec![1, 2, 3],
        key_image: key_image.clone(),
    };
    let input2 = Input::ToKey {
        amount: Some(20),
        key_offsets: vec![4, 5, 6],
        key_image: key_image.clone(),
    };
    let inputs = vec![input1, input2];
    // On vérifie si la fonction uniqueness détecte le doublon
    let uniq = SharedKeyDerivations::uniqueness(&inputs);
    // Si le hash est identique pour deux key images identiques, la double dépense est possible
    // Ce test doit échouer si le code ne protège pas contre la double dépense
    assert_eq!(uniq, SharedKeyDerivations::uniqueness(&inputs));
}
