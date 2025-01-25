 #!/bin/bash

wget -O source_1/items.json https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/items.json
cat source_1/items.json | json_pp > source_1/items_formatted.json

wget -O source_2/items.cdtb.bin.json https://raw.communitydragon.org/latest/game/items.cdtb.bin.json
cat source_2/items.cdtb.bin.json | json_pp > source_2/items_formatted.json

wget -O source_3/items.json http://cdn.merakianalytics.com/riot/lol/resources/latest/en-US/items.json
wget -O source_3/champions.json http://cdn.merakianalytics.com/riot/lol/resources/latest/en-US/champions.json
cat source_3/items.json | json_pp > source_3/items_formatted.json
cat source_3/champions.json | json_pp > source_3/champions_formatted.json

wget -O source_4/121.json https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champions/121.json
cat source_4/121.json | json_pp > source_4/121_formatted.json

# wget -O data/champions/khazix.bin.json https://raw.communitydragon.org/pbe/game/data/characters/khazix/khazix.bin.json
# cat data/champions/khazix.bin.json | json_pp > data/champions/khazix.bin_formated.json

wget -O source_1/perks.json https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perks.json
cat source_1/perks.json | json_pp > source_1/perks_formated.json