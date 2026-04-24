use chrono::{Datelike, Timelike};

use crate::{mccrisk::mcc_risk, payload::FraudRequest};

pub const FEATURE_DIM: usize = 14;

pub struct Normalization {
    pub max_amount: f32,
    pub max_installments: f32,
    pub amount_vs_avg_ratio: f32,
    pub max_minutes: f32,
    pub max_km: f32,
    pub max_tx_count_24h: f32,
    pub max_merchant_avg_amount: f32,
}

pub const NORM: Normalization = Normalization {
    max_amount: 10_000.0,
    max_installments: 12.0,
    amount_vs_avg_ratio: 10.0,
    max_minutes: 1_440.0,
    max_km: 1_000.0,
    max_tx_count_24h: 20.0,
    max_merchant_avg_amount: 10_000.0,
};

pub fn normalize_request(req: &FraudRequest) -> [f32; FEATURE_DIM] {
    let amount = clamp01(req.transaction.amount / NORM.max_amount);
    let installments = clamp01(req.transaction.installments as f32 / NORM.max_installments);

    let amount_vs_avg_raw = if req.customer.avg_amount > 0.0 {
        (req.transaction.amount / req.customer.avg_amount) / NORM.amount_vs_avg_ratio
    } else {
        1.0
    };
    let amount_vs_avg = clamp01(amount_vs_avg_raw);

    let hour_of_day = req.transaction.requested_at.hour() as f32 / 23.0;
    let day_of_week = req
        .transaction
        .requested_at
        .weekday()
        .num_days_from_monday() as f32
        / 6.0;

    let minutes_since_last_tx = req.last_transaction.as_ref().map_or(-1.0, |last| {
        let minutes = (req.transaction.requested_at - last.timestamp)
            .num_minutes()
            .max(0) as f32;
        clamp01(minutes / NORM.max_minutes)
    });

    let km_from_last_tx = req
        .last_transaction
        .as_ref()
        .map_or(-1.0, |last| clamp01(last.km_from_current / NORM.max_km));

    let unknown_merchant = if req
        .customer
        .known_merchants
        .iter()
        .any(|merchant_id| merchant_id == &req.merchant.id)
    {
        0.0
    } else {
        1.0
    };

    let km_from_home = clamp01(req.terminal.km_from_home / NORM.max_km);
    let tx_count_24h = clamp01(req.customer.tx_count_24h as f32 / NORM.max_tx_count_24h);
    let is_online = bool_feature(req.terminal.is_online);
    let card_present = bool_feature(req.terminal.card_present);
    let mcc_risk = mcc_risk(&req.merchant.mcc);
    let merchant_avg_amount = clamp01(req.merchant.avg_amount / NORM.max_merchant_avg_amount);

    [
        amount,
        installments,
        amount_vs_avg,
        hour_of_day,
        day_of_week,
        minutes_since_last_tx,
        km_from_last_tx,
        km_from_home,
        tx_count_24h,
        is_online,
        card_present,
        unknown_merchant,
        mcc_risk,
        merchant_avg_amount,
    ]
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn bool_feature(v: bool) -> f32 {
    if v { 1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::normalize_request;
    use crate::payload::{
        Customer, FraudRequest, LastTransaction, Merchant, Terminal, Transaction,
    };

    fn sample_request() -> FraudRequest {
        FraudRequest {
            id: "tx-1".into(),
            transaction: Transaction {
                amount: 41.12,
                installments: 2,
                requested_at: Utc.with_ymd_and_hms(2026, 3, 11, 18, 45, 53).unwrap(),
            },
            customer: Customer {
                avg_amount: 82.24,
                tx_count_24h: 3,
                known_merchants: vec!["MERC-003".into(), "MERC-016".into()],
            },
            merchant: Merchant {
                id: "MERC-016".into(),
                mcc: "5411".into(),
                avg_amount: 60.25,
            },
            terminal: Terminal {
                is_online: false,
                card_present: true,
                km_from_home: 29.2331036248,
            },
            last_transaction: None,
        }
    }

    #[test]
    fn normalizes_example_payload_in_expected_order() {
        let features = normalize_request(&sample_request());

        assert_eq!(features[0], 0.004112);
        assert!((features[1] - 0.16666667).abs() < 0.0001);
        assert_eq!(features[2], 0.05);
        assert!((features[3] - (18.0 / 23.0)).abs() < 0.0001);
        assert!((features[4] - (2.0 / 6.0)).abs() < 0.0001);
        assert_eq!(features[5], -1.0);
        assert_eq!(features[6], -1.0);
        assert!((features[7] - 0.029233104).abs() < 0.0001);
        assert_eq!(features[8], 0.15);
        assert_eq!(features[9], 0.0);
        assert_eq!(features[10], 1.0);
        assert_eq!(features[11], 0.0);
        assert_eq!(features[12], 0.15);
        assert_eq!(features[13], 0.006025);
    }

    #[test]
    fn uses_sentinel_values_when_last_transaction_is_missing() {
        let features = normalize_request(&sample_request());

        assert_eq!(features[5], -1.0);
        assert_eq!(features[6], -1.0);
    }

    #[test]
    fn clamps_values_and_handles_zero_customer_average() {
        let mut req = sample_request();
        req.transaction.amount = 20_000.0;
        req.transaction.installments = 36;
        req.customer.avg_amount = 0.0;
        req.customer.tx_count_24h = 100;
        req.merchant.avg_amount = 20_000.0;
        req.terminal.km_from_home = 3_000.0;

        let features = normalize_request(&req);

        assert_eq!(features[0], 1.0);
        assert_eq!(features[1], 1.0);
        assert_eq!(features[2], 1.0);
        assert_eq!(features[7], 1.0);
        assert_eq!(features[8], 1.0);
        assert_eq!(features[13], 1.0);
    }

    #[test]
    fn marks_unknown_merchant_and_defaults_unknown_mcc() {
        let mut req = sample_request();
        req.merchant.id = "MERC-999".into();
        req.merchant.mcc = "9999".into();

        let features = normalize_request(&req);

        assert_eq!(features[11], 1.0);
        assert_eq!(features[12], 0.5);
    }

    #[test]
    fn normalizes_last_transaction_fields_when_present() {
        let mut req = sample_request();
        req.last_transaction = Some(LastTransaction {
            timestamp: Utc.with_ymd_and_hms(2026, 3, 11, 14, 58, 35).unwrap(),
            km_from_current: 18.8626479774,
        });

        let features = normalize_request(&req);

        assert!((features[5] - (227.0 / 1440.0)).abs() < 0.0001);
        assert!((features[6] - 0.018862648).abs() < 0.0001);
    }
}
