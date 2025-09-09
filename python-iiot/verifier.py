import os
import time
import base64
import requests

GQL_URL = os.getenv("GQL_URL", "http://localhost:9182/api/v0/graphql")
VERIFY_URL = os.getenv("VERIFY_URL", "http://localhost:9182/api/v0/block/verify-signature")

QUERY = """query($limit:Int!) { 
    SensorEvent(order:{timestamp:ASC}, limit:$limit) {
        _docID
        deviceId 
        timestamp 
        prediction 
        probability 
    }
}"""

QUERYCOMMITS = """query($docID:ID!) { 
    latestCommits(docID:$docID) {
        cid
        signature {
            identity
        }
    }
}"""

last_ts = -1.0

print(f"[verify] Polling {GQL_URL} …")

while True:
    try:
        r = requests.post(
            GQL_URL, json={"query": QUERY, "variables": {"limit": 200}}, timeout=10
        )
        r.raise_for_status()

        sensor_events = r.json().get("data", {}).get("SensorEvent", [])

        new = [e for e in sensor_events if float(e.get("timestamp", -1)) > last_ts]

        if not sensor_events:
            print("[verify] no rows yet")

        for e in sensor_events:
            # The commit block will include an identity based on a secp256k1 key
            r = requests.post(
                GQL_URL, json={"query": QUERYCOMMITS, "variables": {"docID": e["_docID"]}}, timeout=10
            )
            r.raise_for_status()

            commits = r.json().get("data", {}).get("latestCommits", [])

            # We convert the base64 representation of the public key into a hex value.
            hex_value = base64.b64decode(commits[0]["signature"]["identity"]).decode('utf-8')

            # We can then ask DefraDB to validate the signature of the block.
            r = requests.get(
                VERIFY_URL, params={"cid": commits[0]["cid"], "public-key": hex_value}, timeout=10
            )
            r.raise_for_status()

            status = "OK" if (r.status_code == 200) else "FAIL"

            print(
                f"[{status}] ts={e['timestamp']} dev={e['deviceId']} pred={e['prediction']} p={float(e['probability']):.3f}"
            )

            last_ts = max(last_ts, float(e["timestamp"]))

    except Exception as ex:
        print("[verify] error:", ex)

    time.sleep(2)
