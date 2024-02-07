use actix_web::{get, web, HttpResponse, Responder};
use chrono::Utc;
use deadpool_postgres::Pool;

use crate::model::{AccountBalance, AccountStatement, TransactionDetail};

#[get("/clientes/{client_id}/extrato")]
async fn handle(
    client_id_path: web::Path<i32>,
    pool: web::Data<Pool>,
) -> actix_web::Result<impl Responder> {
    let client_id = client_id_path.into_inner();

    if client_id > 5 {
        return Ok(HttpResponse::NotFound().await?);
    }

    let client = pool.get().await.unwrap();
    let stmt = client
        .prepare_cached("SELECT account_limit, balance FROM clients WHERE id = $1")
        .await
        .unwrap();
    let res = client.query_one(&stmt, &[&client_id]).await.unwrap();

    let limit: i32 = res.get(0);
    let balance: i32 = res.get(1);

    let saldo = AccountBalance {
        total: balance,
        data_extrato: Utc::now().to_rfc3339(),
        limite: limit,
    };

    let transaction_stmt = client.prepare_cached(
            r#"SELECT value as valor, operation as tipo, description as descricao, TO_CHAR(created_at, 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') as realizada_em
            FROM transactions WHERE client_id = $1
            ORDER BY id DESC LIMIT 10"#
        ).await.unwrap();

    let last_transactions = client
        .query(&transaction_stmt, &[&client_id])
        .await
        .map(|rows| {
            rows.iter()
                .map(|row| TransactionDetail {
                    valor: row.get(0),
                    tipo: row.get(1),
                    descricao: row.get(2),
                    realizada_em: row.get(3),
                })
                .collect()
        })
        .unwrap();

    Ok(HttpResponse::Ok().json(AccountStatement {
        saldo,
        ultimas_transacoes: last_transactions,
    }))
}
