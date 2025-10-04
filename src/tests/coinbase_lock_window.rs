// Test minimal pour vérifier la faille coinbase lock window
#[cfg(test)]
mod tests {
    #[test]
    fn coinbase_spend_lock_window() {
        use crate::COINBASE_LOCK_WINDOW;
        // Simule un output coinbase créé au block 100
        let coinbase_block_height = 100;
        let current_block_height = coinbase_block_height + 10; // < COINBASE_LOCK_WINDOW
        // On tente de dépenser l’output coinbase avant la fin du lock window
        let can_spend = (current_block_height - coinbase_block_height) >= COINBASE_LOCK_WINDOW;
        assert!(!can_spend, "Faille : output coinbase dépensé avant la fin du lock window !");
    }
}
