use anyhow::Ok;

use crate::{database, entity::AlphaEntity};

pub async fn create(entity: AlphaEntity) -> anyhow::Result<AlphaEntity> {
    let db = &mut database::get().await;
    if check_if_exist(&entity).await? {
        return Ok(entity);
    }
    let result = AlphaEntity::create()
        .id(entity.id)
        .dataset(entity.dataset)
        .crete_time(entity.crete_time)
        .regular(entity.regular)
        .instrument_type(entity.instrument_type)
        .region(entity.region)
        .universe(entity.universe)
        .delay(entity.delay)
        .decay(entity.decay)
        .neutralization(entity.neutralization)
        .truncation(entity.truncation)
        .pasteurization(entity.pasteurization)
        .test_period(entity.test_period)
        .unit_handling(entity.unit_handling)
        .nan_handling(entity.nan_handling)
        .max_trade(entity.max_trade)
        .language(entity.language)
        .visualization(entity.visualization)
        .selection_handling(entity.selection_handling)
        .selection_limit(entity.selection_limit)
        .combo(entity.combo)
        .selection(entity.selection)
        .exec(db)
        .await?;
    Ok(result)
}

/*
"settings":{"decay":0,"delay":1,"instrumentType":"EQUITY","language":"FASTEXPR","maxTrade":"OFF","nanHandling":"OFF","neutralization":"SUBINDUSTRY","pasteurization":"ON","region":"USA","testPeriod":"P0Y0M0D","truncation":0.1,"unitHandling":"VERIFY","universe":"TOP3000","visualization":false}
*/
pub async fn check_if_exist(entity: &AlphaEntity) -> anyhow::Result<bool> {
    let db = &mut database::get().await;
    let result = AlphaEntity::filter(
    AlphaEntity::fields().regular().eq(entity.regular.clone())
            .and(AlphaEntity::fields().instrument_type().eq(entity.instrument_type.clone()))
            .and(AlphaEntity::fields().region().eq(entity.region.clone()))
            .and(AlphaEntity::fields().universe().eq(entity.universe.clone()))
            .and(AlphaEntity::fields().delay().eq(entity.delay))
            .and(AlphaEntity::fields().decay().eq(entity.decay))
            .and(AlphaEntity::fields().neutralization().eq(entity.neutralization.clone()))
            .and(AlphaEntity::fields().truncation().eq(entity.truncation))
            .and(AlphaEntity::fields().pasteurization().eq(entity.pasteurization.clone()))
            .and(AlphaEntity::fields().unit_handling().eq(entity.unit_handling.clone()))
            .and(AlphaEntity::fields().nan_handling().eq(entity.nan_handling.clone()))
            .and(AlphaEntity::fields().max_trade().eq(entity.max_trade.clone()))
            .and(AlphaEntity::fields().language().eq(entity.language.clone()))
            .and(AlphaEntity::fields().test_period().eq(entity.test_period.clone()))
            .and(AlphaEntity::fields().visualization().eq(entity.visualization))
    )
    .exec(db)
    .await?;
    Ok(result.len() > 0)
}
