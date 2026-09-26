# Premiere Pro clip markers in XMP

Research notes for issue #9 (markers). Not yet verified in Premiere Pro by hand.

## Structure

Markers live in `xmpDM:Tracks` (rdf:Bag of Track structs): `trackName`, `trackType`, `frameRate`,
`markers` (rdf:Seq of Marker structs). Marker fields (Adobe xmp-docs, Marker.md): `startTime`,
`duration` (FrameCount, default 0), `name`, `comment`, `type`, `location`/`target` (WebLink),
`cuePointType` + `cuePointParams` (Seq of `{key, value}`), `speaker`/`probability` (Speech).
Premiere also writes `xmpDM:guid` (not in the spec).

`frameRate` is `f<rate>` or `f<num>s<den>`; `startTime`/`duration` are integers in that rate.
frename's InOut marker uses `f1000` (milliseconds) — keep that for new tracks. Premiere snaps to
frames on import (up to one frame of precision loss).

## Colored comment marker

```xml
<rdf:li rdf:parseType="Resource">
 <xmpDM:trackName>Comment</xmpDM:trackName>
 <xmpDM:trackType>Comment</xmpDM:trackType>
 <xmpDM:frameRate>f1000</xmpDM:frameRate>
 <xmpDM:markers><rdf:Seq>
  <rdf:li rdf:parseType="Resource">
   <xmpDM:startTime>83500</xmpDM:startTime>
   <xmpDM:duration>2000</xmpDM:duration>
   <xmpDM:name>Take 3</xmpDM:name>
   <xmpDM:comment>line one&#xD;line two</xmpDM:comment>
   <xmpDM:guid>c19c0922-775d-465c-917b-6a7c3a5c0902</xmpDM:guid>
   <xmpDM:cuePointParams><rdf:Seq>
    <rdf:li rdf:parseType="Resource"><xmpDM:key>marker_guid</xmpDM:key>
      <xmpDM:value>c19c0922-775d-465c-917b-6a7c3a5c0902</xmpDM:value></rdf:li>
    <rdf:li rdf:parseType="Resource"><xmpDM:key>keywordExtDVAv1_e8dd1f3f-12f9-41f0-8672-e3296ce19cec</xmpDM:key>
      <xmpDM:value>{"color":4281740498,"index":0,"name":"","payload":""}</xmpDM:value></rdf:li>
   </rdf:Seq></xmpDM:cuePointParams>
  </rdf:li>
 </rdf:Seq></xmpDM:markers>
</rdf:li>
```

Fresh UUID per marker, and a separate one for the `keywordExtDVAv1_` key. Line breaks as CR.

## Color (medium-high confidence)

`cuePointParams` entry `keywordExtDVAv1_<uuid>` = JSON `{"color":<uint32 0xAABBGGRR>}`. Evidence: a
Premiere 2018 / Media Encoder 2020 sidecar with exactly this value; Adobe XMP SDK defines the
`keywordExtDVAv1_` prefix; two open-source writers (better-markers OBS plugin, ninjav_xml_writer).

| Color | Value |
|---|---|
| Green (default) | omit the key |
| Red | 4281740498 |
| Orange | 4280578025 |
| Yellow | 4281049552 |
| White | 4294967295 |
| Blue | 4294741314 |
| Cyan | 4292277273 |
| Lavender | 4289825711 |
| Magenta | 4294902015 |

Newer Premiere versions may differ: capture a marker Premiere writes itself to confirm.

## Gotchas

- Premiere reads file XMP on import and caches it: re-import (or new project) to see changes.
- With "Write clip markers to XMP" on, Premiere writes its markers back — round-trip reading must
  accept any `frameRate` (`f48000`, `f254016000000`, `fNsD`), attribute and `parseType="Resource"`
  forms, and GUIDs with an `xmp:id:` prefix.
- Leave foreign tracks alone (Premiere/AME write `Markers`/`Cue` tracks).
- Types other than Comment (Chapter, WebLink, FLVCuePoint) have no verified Premiere samples:
  support Comment first.

## Sources

- https://github.com/adobe/xmp-docs/blob/master/XMPNamespaces/XMPDataTypes/Marker.md
- https://github.com/hfiguiere/dng_sdk/blob/master/xmp/toolkit/XMPExtensions/XMPMarker/source/MarkerImpl.cpp
- https://github.com/sjaanii/jeanneshaw.nl/blob/main/assets/work/Modebelofte/Kira%20Goodey.webm.xmp
- https://github.com/przxmus/better-markers/blob/main/src/bm-xmp-sidecar-writer.cpp
- https://github.com/QianYuuRiri/ninjav_xml_writer/blob/main/app_native_timebase.py
- https://github.com/andrewconnell/node-xmp-marker/tree/master/test/samples
- https://github.com/SubtitleEdit/subtitleedit/blob/main/src/libse/SubtitleFormats/Xmp.cs
- https://helpx.adobe.com/premiere/desktop/organize-media/edit-metadata/edit-xmp-metadata.html
