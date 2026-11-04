// Similarite de contenu entre deux titres : TF-IDF + cosinus, calcule sur le
// corpus fourni a chaque requete (quelques centaines de titres au plus a
// cette echelle, pas besoin d'un index pre-calcule). C'est la moitie
// "filtrage par contenu" du systeme hybride (voir recommendations::handlers) :
// une technique standard de recommandation/recherche d'information, pas une
// heuristique ad hoc.

use std::collections::{HashMap, HashSet};

// Tokenisation volontairement simple (minuscules, mots alphanumeriques) :
// les titres de ce catalogue melangent francais/anglais, une tokenisation
// NLP specifique a une langue ferait plus de mal que de bien ici.
fn tokenize(title: &str) -> Vec<String> {
    title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.len() > 1)
        .map(str::to_string)
        .collect()
}

pub struct TfIdfModel {
    // idf(terme) = ln((N + 1) / (df + 1)) + 1 (variante "lissee" classique,
    // evite une division par zero et un score nul pour un terme present
    // dans tous les documents).
    idf: HashMap<String, f64>,
}

impl TfIdfModel {
    pub fn fit<'a>(documents: impl Iterator<Item = &'a str>) -> Self {
        let mut doc_count: usize = 0;
        let mut document_frequency: HashMap<String, usize> = HashMap::new();

        for doc in documents {
            doc_count += 1;
            let unique_terms: HashSet<String> = tokenize(doc).into_iter().collect();
            for term in unique_terms {
                *document_frequency.entry(term).or_insert(0) += 1;
            }
        }

        let idf = document_frequency
            .into_iter()
            .map(|(term, df)| {
                let weight = ((doc_count as f64 + 1.0) / (df as f64 + 1.0)).ln() + 1.0;
                (term, weight)
            })
            .collect();

        TfIdfModel { idf }
    }

    fn vector(&self, title: &str) -> HashMap<String, f64> {
        let tokens = tokenize(title);
        if tokens.is_empty() {
            return HashMap::new();
        }

        let mut term_frequency: HashMap<String, f64> = HashMap::new();
        for token in &tokens {
            *term_frequency.entry(token.clone()).or_insert(0.0) += 1.0;
        }

        let token_count = tokens.len() as f64;
        term_frequency
            .into_iter()
            .map(|(term, count)| {
                let tf = count / token_count;
                let idf = self.idf.get(&term).copied().unwrap_or(1.0);
                (term, tf * idf)
            })
            .collect()
    }

    // Cosinus entre les vecteurs TF-IDF de deux titres, dans [0, 1] (les poids
    // sont tous positifs ici, pas besoin de gerer un cosinus negatif).
    pub fn similarity(&self, a: &str, b: &str) -> f64 {
        let vec_a = self.vector(a);
        let vec_b = self.vector(b);

        let dot: f64 = vec_a.iter().filter_map(|(term, weight)| vec_b.get(term).map(|other| weight * other)).sum();
        let norm_a = vec_a.values().map(|w| w * w).sum::<f64>().sqrt();
        let norm_b = vec_b.values().map(|w| w * w).sum::<f64>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_titles_score_highest() {
        let titles = ["Dracula 1931", "Nosferatu", "Sherlock Holmes"];
        let model = TfIdfModel::fit(titles.iter().copied());

        let same = model.similarity("Dracula 1931", "Dracula 1931");
        let unrelated = model.similarity("Dracula 1931", "Sherlock Holmes");
        assert!(same > unrelated);
        assert!((same - 1.0).abs() < 1e-9);
    }

    #[test]
    fn disjoint_titles_score_zero() {
        let titles = ["Dracula", "Popeye le marin"];
        let model = TfIdfModel::fit(titles.iter().copied());
        assert_eq!(model.similarity("Dracula", "Popeye le marin"), 0.0);
    }
}
