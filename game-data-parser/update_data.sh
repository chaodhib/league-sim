 #!/bin/bash

wget -O source_2/items.cdtb.bin.json https://raw.communitydragon.org/latest/game/items.cdtb.bin.json
cat source_2/items.cdtb.bin.json | json_pp > source_2/items_formatted.json

# wget -O source_3/items.json http://cdn.merakianalytics.com/riot/lol/resources/latest/en-US/items.json
# wget -O source_3/champions.json http://cdn.merakianalytics.com/riot/lol/resources/latest/en-US/champions.json
# cat source_3/items.json | json_pp > source_3/items_formatted.json
# cat source_3/champions.json | json_pp > source_3/champions_formatted.json

# wget -O source_3/champions/Khazix.json http://cdn.merakianalytics.com/riot/lol/resources/latest/en-US/champions/Khazix.json

cp $MERAKI_LOL_STATIC_DATA_PATH/items.json source_3/items.json
cp $MERAKI_LOL_STATIC_DATA_PATH/champions.json source_3/champions.json
cp $MERAKI_LOL_STATIC_DATA_PATH/champions/Khazix.json source_3/champions/Khazix.json