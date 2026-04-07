from brain.database import IsStatsEntity, AlphaEntity
from brain.logger_util import logger


class IsStatsDao:
    @staticmethod
    def insert(item: dict):
        alpha_id = item["alpha_id"]
        existed = (
            IsStatsEntity.select().where(IsStatsEntity.alpha_id == alpha_id).exists()
        )
        if existed:
            logger.warning(f"is_stats_entity exists: {alpha_id}")
            return
        new_row = IsStatsEntity.create(
            alpha_id=alpha_id,
            pnl=item["pnl"],
            book_size=item["book_size"],
            long_count=item["long_count"],
            short_count=item["short_count"],
            turnover=item["turnover"],
            returns=item["returns"],
            drawdown=item["drawdown"],
            margin=item["margin"],
            sharpe=item["sharpe"],
            fitness=item["fitness"],
            start_date=item["start_date"],
        )
        logger.debug(f"add to is_stats_entity: {new_row.alpha_id}")

    @staticmethod
    def query_by_dataset_id(dataset_id: str) -> list:
        query = (
            IsStatsEntity.select()
            .join(AlphaEntity, on=(IsStatsEntity.alpha_id == AlphaEntity.alpha_id))
            .where(AlphaEntity.dataset_id == dataset_id)
        )
        return list(query)
