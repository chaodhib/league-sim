 #!/bin/bash

wget -O source_1/items.json https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/items.json
cat source_1/items.json | json_pp > source_1/items_formatted.json

wget -O source_2/items.cdtb.bin.json https://raw.communitydragon.org/latest/game/items.cdtb.bin.json
cat source_2/items.cdtb.bin.json | json_pp > source_2/items_formatted.json

wget -O source_3/items.json http://cdn.merakianalytics.com/riot/lol/resources/latest/en-US/items.json
wget -O source_3/champions.json http://cdn.merakianalytics.com/riot/lol/resources/latest/en-US/champions.json
cat source_3/items.json | json_pp > source_3/items_formatted.json
cat source_3/champions.json | json_pp > source_3/champions_formatted.json