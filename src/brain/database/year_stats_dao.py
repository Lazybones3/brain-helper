from brain.database import YearStatsEntity
from brain.logger_util import logger


class YearStatsDao:
    @staticmethod
    def insert(year_stats_dict: dict):
        alpha_id = year_stats_dict['alpha_id']
        year = year_stats_dict['year']
        existed = YearStatsEntity.select().where((YearStatsEntity.alpha_id==alpha_id) & (YearStatsEntity.year == year)).exists()
        if existed:
            logger.warning(f"year_stats_entity exists: {alpha_id}, {year}")
            return
        new_row = YearStatsEntity.create(
            year = year,
            pnl = year_stats_dict['pnl'],
            book_size = year_stats_dict['book_size'],
            long_count = year_stats_dict['long_count'],
            short_count = year_stats_dict['short_count'],
            turnover = year_stats_dict['turnover'],
            sharpe = year_stats_dict['sharpe'],
            returns = year_stats_dict['returns'],
            drawdown = year_stats_dict['drawdown'],
            margin = year_stats_dict['margin'],
            fitness = year_stats_dict['fitness'],
            stage = year_stats_dict['stage'],
            alpha_id = alpha_id,
        )
        logger.debug(f"add to year_stats_entity: {new_row.alpha_id}, {new_row.year}")
