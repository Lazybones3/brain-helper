from brain.database import RiskNeutralizedEndity
from brain.logger_util import logger


class RiskNeutralizedDao:
    @staticmethod
    def insert(item: dict):
        alpha_id = item['alpha_id']
        existed = RiskNeutralizedEndity.select().where(RiskNeutralizedEndity.alpha_id==alpha_id).exists()
        if existed:
            logger.warning(f"risk_neutralized_entity exists: {alpha_id}")
            return
        new_row = RiskNeutralizedEndity.create(
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
        logger.debug(f"add to risk_neutralized_entity: {new_row.alpha_id}")