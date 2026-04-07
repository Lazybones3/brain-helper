from mcp.server.fastmcp import FastMCP
from brain.api import BrainApi

mcp = FastMCP()
api = BrainApi()


@mcp.tool()
async def get_datafields(
    dataset_id: str,
    region: str = "USA",
    universe: str = "TOP3000"
):
    """Get specific data fields you can use in alpha formula
    Args:
        dataset_id: Specific dataset ID must provided by user
        region: Market region (e.g., "USA")
        universe: Universe of stocks (e.g., "TOP3000")
    Returns:
        Available data fields
    """
    try:
        datafields_df = api.get_datafields(region=region, universe=universe)
        filtered_datafields_df = datafields_df[
            (datafields_df["dataset_id"] == dataset_id)
        ].reset_index(drop=True)
        return filtered_datafields_df[["id", "description", "type", "subcategory_name"]].to_json(orient='records')
    except Exception as e:
        return {"error": f"An unexpected error occurred: {str(e)}"}

@mcp.tool()
async def get_operators():
    """Get available operators for alpha creation.
    Returns:
        Available operators
    """
    try:
        operators = api.get_operators()
        operators_info = operators[["definition", "category", "description"]]
        return operators_info.to_json(orient='records')
    except Exception as e:
        return {"error": f"An unexpected error occurred: {str(e)}"}


def main():
    mcp.run()


if __name__ == "__main__":
    main()
