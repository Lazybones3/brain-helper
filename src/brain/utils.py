import re
import pandas as pd


def dataframe_to_md_table(df: pd.DataFrame) -> str:
    md_table = df.to_markdown(index=False)
    clean_md = re.sub(r' +\| +', '|', md_table)
    clean_md = re.sub(r' +\|', '|', clean_md, flags=re.M)
    clean_md = re.sub(r'\| +', '|', clean_md, flags=re.M)
    return clean_md
