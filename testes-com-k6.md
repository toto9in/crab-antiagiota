# Melhorar Detection Score

  ## Resumo

  A detecção já está quase igual ao gabarito: 14 FP, 3 FN, E=23. O maior ganho vem de eliminar os FN, porque cada um pesa 3x. A hipótese mais forte no código atual é que o HNSW do pgvector está retornando
  vizinhos aproximados, enquanto o gabarito foi gerado com k-NN exato.

  ## Mudanças Prioritárias

  - Primeiro experimento: aumentar recall do HNSW por query com SET LOCAL hnsw.ef_search = 100, depois testar 200 e 400.
  - Segundo experimento: recriar o índice HNSW com maior qualidade, por exemplo WITH (m = 32, ef_construction = 128), medindo memória/startup.
  - Terceiro experimento: rodar uma variante de busca exata, desabilitando o índice só para comparação offline, para confirmar se os 17 erros são causados por ANN.
  - Se a busca exata zerar ou reduzir muito os erros, escolher o menor ef_search que preserve a melhora sem piorar demais o p99.
  - Evitar mudar threshold às cegas: approved = fraud_score < 0.6 é parte da regra, e alterar isso pode trocar poucos FN por muitos FP.


primeiro teste
{
  "expected": {
    "total": 14500,
    "fraud_count": 4812,
    "legit_count": 9688,
    "fraud_rate": 0.3319,
    "legit_rate": 0.6681,
    "edge_case_count": 157,
    "edge_case_rate": 0.0108
  },
  "p99": "11.57ms",
  "scoring": {
    "breakdown": {
      "false_positive_detections": 14,
      "false_negative_detections": 3,
      "true_positive_detections": 4719,
      "true_negative_detections": 9503,
      "http_errors": 0
    },
    "failure_rate": "0.12%",
    "weighted_errors_E": 23,
    "error_rate_epsilon": 0.001615,
    "p99_score": {
      "value": 1936.63,
      "cut_triggered": false
    },
    "detection_score": {
      "value": 2377.69,
      "rate_component": 2791.75,
      "absolute_penalty": -414.06,
      "cut_triggered": false
    },
    "final_score": 4314.32
  }
}

segundo teste
{
  "expected": {
    "total": 14500,
    "fraud_count": 4812,
    "legit_count": 9688,
    "fraud_rate": 0.3319,
    "legit_rate": 0.6681,
    "edge_case_count": 157,
    "edge_case_rate": 0.0108
  },
  "p99": "13.53ms",
  "scoring": {
    "breakdown": {
      "false_positive_detections": 14,
      "false_negative_detections": 3,
      "true_positive_detections": 4716,
      "true_negative_detections": 9494,
      "http_errors": 0
    },
    "failure_rate": "0.12%",
    "weighted_errors_E": 23,
    "error_rate_epsilon": 0.001617,
    "p99_score": {
      "value": 1868.85,
      "cut_triggered": false
    },
    "detection_score": {
      "value": 2377.32,
      "rate_component": 2791.39,
      "absolute_penalty": -414.06,
      "cut_triggered": false
    },
    "final_score": 4246.17
  }
}
