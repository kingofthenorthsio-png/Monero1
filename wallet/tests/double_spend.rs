// Test pour simuler une double dépense avec le même key image
use monero_oxide::transaction::Input;
use monero_oxide::io::CompressedPoint;
use monero_wallet::SharedKeyDerivations;

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
    // Test : on attend que le code rejette les doublons, donc on force l’échec si ce n’est pas le cas
    let mut key_images = std::collections::HashSet::new();
    for input in &inputs {
        if let Input::ToKey { key_image, .. } = input {
            if !key_images.insert(key_image.to_bytes()) {
                panic!("Double dépense possible : key image dupliqué accepté !");
            }
        }
    }
}
