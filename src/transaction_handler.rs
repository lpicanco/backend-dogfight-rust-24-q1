use crate::model::Transaction;
use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use validator::Validate;

#[post("/clientes/{client_id}/transacoes")]
async fn handle(
    client_id_path: web::Path<i32>,
    transaction: web::Json<Transaction>,
    pool: web::Data<Pool>,
) -> actix_web::Result<impl Responder> {
    let client_id = client_id_path.into_inner();
    if let Err(errors) = transaction.validate() {
        return Ok(HttpResponse::UnprocessableEntity().json(errors));
    }

    if transaction.valor.unwrap().ceil() != transaction.valor.unwrap() {
        return Ok(HttpResponse::UnprocessableEntity().json("Valor deve ser um número inteiro"));
    }

    if client_id > 5 {
        return Ok(HttpResponse::NotFound().await?);
    }

    let value = transaction.valor.unwrap() as i32;

    return match transaction.tipo.clone().unwrap().as_str() {
        "d" => Ok(insert_debit(transaction, client_id, value * -1, pool).await),
        "c" => Ok(insert_credit(transaction, client_id, value, pool).await),
        _ => return Ok(HttpResponse::UnprocessableEntity().json("Tipo de transação inválido")),
    };
}

async fn insert_debit(
    transaction: web::Json<Transaction>,
    client_id: i32,
    value: i32,
    pool: web::Data<Pool>,
) -> HttpResponse {
    let client = pool.get().await.unwrap();
    let stmt = client
        .prepare_cached(
            r#"
        WITH 
        upd AS (
            UPDATE clients
                SET    balance = balance + $2
                WHERE  id = $1 and balance + $2 >= account_limit * -1
                RETURNING balance, account_limit
        ), 
        ins AS (
            INSERT INTO transactions (client_id, value, operation, description)
            SELECT $1, $3, $4, $5  FROM upd
        )
        SELECT balance, account_limit FROM upd;
        "#,
        )
        .await
        .unwrap();

    client
        .query_opt(
            &stmt,
            &[
                &client_id,
                &value,
                &(transaction.valor.unwrap() as i32),
                &transaction.tipo,
                &transaction.descricao,
            ],
        )
        .await
        .map(|row| match row {
            None => HttpResponse::UnprocessableEntity().finish(),
            Some(row) => {
                let balance: i32 = row.get(0);
                let limit: i32 = row.get(1);
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(format!("{{\"limite\" : {limit}, \"saldo\" : {balance}}}"))
            }
        })
        .unwrap()
}

async fn insert_credit(
    transaction: web::Json<Transaction>,
    client_id: i32,
    value: i32,
    pool: web::Data<Pool>,
) -> HttpResponse {
    let client = pool.get().await.unwrap();
    let stmt = client
        .prepare_cached(
            r#"
        WITH
        ins AS (
            INSERT INTO transactions (client_id, value, operation, description)
            VALUES($1, $3, $4, $5)
        )         
        UPDATE clients
                SET    balance = balance + $2
                WHERE  id = $1
                RETURNING balance, account_limit
        "#,
        )
        .await
        .unwrap();

    client
        .query_opt(
            &stmt,
            &[
                &client_id,
                &value,
                &(transaction.valor.unwrap() as i32),
                &transaction.tipo,
                &transaction.descricao,
            ],
        )
        .await
        .map(|row| match row {
            None => HttpResponse::UnprocessableEntity().finish(),
            Some(row) => {
                let balance: i32 = row.get(0);
                let limit: i32 = row.get(1);
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(format!("{{\"limite\" : {limit}, \"saldo\" : {balance}}}"))
            }
        })
        .unwrap()
}
