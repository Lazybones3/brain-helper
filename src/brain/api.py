import time
from typing import Literal, Optional
import pandas as pd
from requests.exceptions import RequestException

import ace_lib as ace
from brain.logger_util import logger


class BrainApi:
    def __init__(self) -> None:
        self.session = ace.start_session()
    
    def check_session_and_relogin(self):
        self.session = ace.check_session_and_relogin(self.session)

    def check_session_timeout(self) -> int:
        return ace.check_session_timeout(self.session)
    
    def get_datasets(
        self,
        instrument_type: str = "EQUITY",
        region: str = "USA",
        delay: int = 1,
        universe: str = "TOP3000",
        theme: Optional[bool] = None,
    ) -> pd.DataFrame:
        return ace.get_datasets(self.session, instrument_type=instrument_type, region=region, delay=delay, universe=universe, theme=theme)

    def get_datafields(
        self,
        instrument_type: str = "EQUITY",
        region: str = "USA",
        delay: int = 1,
        universe: str = "TOP3000",
        search: str = "",
    ) -> pd.DataFrame:
        return ace.get_datafields(self.session, instrument_type=instrument_type, region=region, delay=delay, universe=universe, search=search)

    def generate_alpha(
        self,
        regular: Optional[str] = None,
        selection: Optional[str] = None,
        combo: Optional[str] = None,
        alpha_type: Literal["REGULAR", "SUPER"] = "REGULAR",
        region: str = "USA",
        universe: str = "TOP3000",
        delay: Literal[0, 1] = 1,
        decay: int = 0,
        neutralization: str = "INDUSTRY",
        truncation: float = 0.08,
        pasteurization: Literal["ON", "OFF"] = "ON",
        test_period: str = "P0Y0M0D",
        unit_handling: Literal["VERIFY"] = "VERIFY",
        nan_handling: Literal["ON", "OFF"] = "OFF",
        max_trade: Literal["ON", "OFF"] = "OFF",
        selection_handling: str = "POSITIVE",
        selection_limit: int = 100,
        visualization: bool = False,
    ) -> dict:
        return ace.generate_alpha(regular=regular, selection=selection, combo=combo, alpha_type=alpha_type, region=region, universe=universe, delay=delay, decay=decay, neutralization=neutralization, truncation=truncation, pasteurization=pasteurization, test_period=test_period, unit_handling=unit_handling, nan_handling=nan_handling, max_trade=max_trade, selection_handling=selection_handling, selection_limit=selection_limit, visualization=visualization)
    
    def simulate_alpha_list_multi(
        self,
        alpha_list: list,
        limit_of_concurrent_simulations: int = 3,
        limit_of_multi_simulations: int = 3,
    ) -> list:
        result = []
        try:
            result = ace.simulate_alpha_list_multi(self.session, alpha_list=alpha_list, limit_of_concurrent_simulations=limit_of_concurrent_simulations, limit_of_multi_simulations=limit_of_multi_simulations)
        except RequestException as e:
            logger.error(f"simulate_alpha_list_multi exception: {e}")
            time.sleep(10)
        return result

    def get_operators(self) -> pd.DataFrame:
        return ace.get_operators(self.session)

    def get_alpha_yearly_stats(self, alpha_id: str) -> pd.DataFrame:
        return ace.get_alpha_yearly_stats(self.session, alpha_id=alpha_id)

    def get_alpha_pnl(self, alpha_id: str) -> pd.DataFrame:
        return ace.get_alpha_pnl(self.session, alpha_id=alpha_id)

    def get_simulation_result_json(self, alpha_id: str) -> dict:
        return ace.get_simulation_result_json(self.session, alpha_id=alpha_id)

    def get_check_submission(self, alpha_id: str) -> pd.DataFrame:
        return ace.get_check_submission(self.session, alpha_id=alpha_id)
