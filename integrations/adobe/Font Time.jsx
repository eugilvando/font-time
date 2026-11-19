#target bridge
// Install through Adobe Bridge > Preferences > Startup Scripts.
(function () {
    if (BridgeTalk.appName !== "bridge" || $.global.fontTimeBridge) return;
    var root = new Folder(__FONT_TIME_DIRECTORY__);
    var hosts = ["indesign", "photoshop", "illustrator"];
    var pending = {};
    function publish(host, body) {
        var dest = new File(root.fsName + "/" + host + ".txt");
        var tmp = new File(root.fsName + "/" + host + ".tmp");
        tmp.encoding = "UTF-8";
        if (!tmp.open("w")) return;
        tmp.write("FT1\n" + body);
        tmp.close();
        if (dest.exists && !dest.remove()) return;
        tmp.rename(host + ".txt");
    }
    // Executed inside the host. Only font identifiers leave the document.
    function usedFonts() {
        var found = {}, lines = [];
        function add(name) {
            if (!name) return;
            var key = encodeURIComponent(String(name));
            if (!found["f" + key]) { found["f" + key] = true; lines.push(key); }
        }
        function psLayers(layers, docId) {
            for (var i = 0; i < layers.length; i++) {
                var l = layers[i];
                if (l.typename === "LayerSet") { psLayers(l.layers, docId); continue; }
                if (l.kind !== LayerKind.TEXT) continue;
                var ref = new ActionReference();
                ref.putIdentifier(stringIDToTypeID("layer"), l.id);
                ref.putIdentifier(stringIDToTypeID("document"), docId);
                var text = executeActionGet(ref).getObjectValue(stringIDToTypeID("textKey"));
                var ranges = text.getList(stringIDToTypeID("textStyleRange"));
                for (var j = 0; j < ranges.count; j++) {
                    var style = ranges.getObjectValue(j).getObjectValue(stringIDToTypeID("textStyle"));
                    var key = stringIDToTypeID("fontPostScriptName");
                    if (style.hasKey(key)) add(style.getString(key));
                }
                add(l.textItem.font);
            }
        }
        for (var d = 0; d < app.documents.length; d++) {
            var doc = app.documents[d];
            if (BridgeTalk.appName === "indesign") {
                for (var n = 0; n < doc.fonts.length; n++) add(doc.fonts[n].postscriptName);
            } else if (BridgeTalk.appName === "photoshop") {
                psLayers(doc.layers, doc.id);
            } else if (BridgeTalk.appName === "illustrator") {
                // The font inventory in XMP retains PostScript names for unavailable faces.
                var ns = new Namespace("http://ns.adobe.com/xap/1.0/sType/Font#");
                var xml = new XML(doc.XMPString);
                var names = xml.descendants(new QName(ns, "fontName"));
                for (var k = 0; k < names.length(); k++) add(names[k].toString());
                for (var f = 0; f < doc.textFrames.length; f++) {
                    var runs = doc.textFrames[f].textRanges;
                    for (var j = 0; j < runs.length; j++) {
                        try { add(runs[j].characterAttributes.textFont.name); } catch (_) {}
                    }
                }
            }
        }
        return lines.join("\n");
    }
    function poll(host) {
        if (pending[host]) return;
        var target = BridgeTalk.getSpecifier(host);
        if (!target || !BridgeTalk.isRunning(target)) { publish(host, ""); return; }
        var msg = new BridgeTalk();
        msg.target = target;
        msg.body = "(" + usedFonts.toString() + ")()";
        msg.onResult = function (reply) { delete pending[host]; publish(host, reply.body || ""); };
        msg.onTimeout = msg.onError = function () { delete pending[host]; };
        pending[host] = msg;
        msg.timeout = 12;
        if (!msg.send()) delete pending[host];
    }
    $.global.fontTimeBridge = function () {
        var marker = new File(root.fsName + "/enabled");
        if (marker.exists && new Date().getTime() - marker.modified.getTime() < 45000) {
            for (var i = 0; i < hosts.length; i++) poll(hosts[i]);
        }
        return true;
    };
    app.scheduleTask("$.global.fontTimeBridge()", 10000, true);
}());
