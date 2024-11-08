<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.11.0" name="tiles" tilewidth="16" tileheight="16" tilecount="64" columns="4">
 <image source="tiles.png" width="64" height="256"/>
 <tile id="6" type="TileBundle">
  <properties>
   <property name="terrain" propertytype="terrain" value="grass"/>
  </properties>
 </tile>
 <tile id="12" type="TileBundle"/>
 <tile id="22" type="TileBundle">
  <properties>
   <property name="terrain" propertytype="terrain" value="tree"/>
  </properties>
 </tile>
 <wangsets>
  <wangset name="Terrain" type="corner" tile="-1"/>
 </wangsets>
</tileset>
