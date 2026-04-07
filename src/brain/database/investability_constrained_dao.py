from brain.database import InvestabilityConstrainedEntity
from brain.logger_util import logger


class InvestabilityConstrainedDao:
    @staticmethod
    def insert(item: dict):
        alpha_id = item['alpha_id']
        existed = InvestabilityConstrainedEntity.select().where(InvestabilityConstrainedEntity.alpha_id==alpha_id).exists()
        if existed:
            logger.warning(f"investability_constrained_entity exists: {alpha_id}")
            return
        new_row = InvestabilityConstrainedEntity.create(
            alpha_id = alpha_id,
            pnl = item['pnl'],
            book_size = item['book_size'],
            long_count = item['long_count'],
            short_count = item['short_count'],
            turnover = item['turnover'],
            returns = item['returns'],
            drawdown = item['drawdown'],
            margin = item['margin'],
            sharpe = item['sharpe'],
            fitness = item['fitness'],
        )
        logger.debug(f"add to investability_constrained_entity: {new_row.alpha_id}")