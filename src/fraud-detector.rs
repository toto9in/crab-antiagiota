use pgvector::Vector;
use sqlx::PgPool;

use crate::{
    normalization::{FEATURE_DIM, normalize_request},
    payload::FraudRequest,
};

#[derive(Clone)]
pub struct FraudDetector {
    pool: PgPool,
}

#[derive(Debug)]
pub struct FraudAnalysis {
    pub approved: bool,
    pub fraud_score: f32,
}

impl FraudDetector {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn analyze(&self, req: &FraudRequest) -> Result<FraudAnalysis, sqlx::Error> {
        let features = normalize_request(req);
        let nearest_labels = self.closest_labels(&features).await?;
        let analysis = score_from_labels(&nearest_labels)?;

        Ok(analysis)
    }

    async fn closest_labels(
        &self,
        features: &[f32; FEATURE_DIM],
    ) -> Result<Vec<String>, sqlx::Error> {
        let vector = Vector::from(features.to_vec());

        let labels = sqlx::query_scalar::<_, String>(
            "SELECT label FROM vetores_rinha ORDER BY vector <-> $1 LIMIT 5",
        )
        .bind(vector)
        .fetch_all(&self.pool)
        .await?;

        Ok(labels)
    }
}

fn score_from_labels(labels: &[String]) -> Result<FraudAnalysis, sqlx::Error> {
    if labels.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }

    let fraud_count = labels
        .iter()
        .filter(|label| label.as_str() == "fraud")
        .count();
    let fraud_score = fraud_count as f32 / 5.0;
    let approved = fraud_score < 0.6;

    let analysis = FraudAnalysis {
        approved,
        fraud_score,
    };

    Ok(analysis)
}

#[cfg(test)]
mod tests {
    use super::score_from_labels;

    #[test]
    fn rejects_score_at_threshold() {
        let labels = vec![
            "fraud".to_string(),
            "fraud".to_string(),
            "fraud".to_string(),
            "legit".to_string(),
            "legit".to_string(),
        ];

        let analysis = score_from_labels(&labels).unwrap();

        assert_eq!(analysis.fraud_score, 0.6);
        assert!(!analysis.approved);
    }

    #[test]
    fn approves_below_threshold() {
        let labels = vec![
            "fraud".to_string(),
            "fraud".to_string(),
            "legit".to_string(),
            "legit".to_string(),
            "legit".to_string(),
        ];

        let analysis = score_from_labels(&labels).unwrap();

        assert_eq!(analysis.fraud_score, 0.4);
        assert!(analysis.approved);
    }

    #[test]
    fn fails_when_no_neighbors_are_found() {
        let err = score_from_labels(&[]).unwrap_err();

        assert!(matches!(err, sqlx::Error::RowNotFound));
    }
}
