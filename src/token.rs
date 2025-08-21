use rand::Rng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

#[derive(Clone)] // Додаємо Clone trait
pub struct TokenGenerator {
    length: usize,
}

impl TokenGenerator {
    pub fn new() -> Self {
        Self { length: 6 }
    }
#[allow(dead_code)]
    pub fn with_length(length: usize) -> Self {
        Self { length }
    }

    pub fn generate(&self) -> String {
        let mut rng = rand::thread_rng();
        (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let generator = TokenGenerator::new();
        let token = generator.generate();
        
        assert_eq!(token.len(), 6);
        assert!(token.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_custom_length() {
        let generator = TokenGenerator::with_length(10);
        let token = generator.generate();
        
        assert_eq!(token.len(), 10);
    }
}
