import os
import json
import time
import sys

import requests
import serial

from dotenv import load_dotenv


load_dotenv()

SERIAL_PORT = os.getenv("SERIAL_PORT", "/dev/ttyACM0")
BAUD = int(os.getenv("BAUD", "115200"))
GQL_URL = os.getenv("GQL_URL", "http://localhost:9181/api/v0/graphql")


def gql_create(event):
    # Your server exposes create_SensorEvent(input: [SensorEventMutationInputArg!]!)
    #
    # DefraDB, by default, will sign the document using the node's private key.
    # The signature along with the public key is stored in the document's Merkle CRDT blocks
    # with every update. The blocks are identified by their content ID (hash) which 
    # ensures a verifiable history of change.
    return {
        "query": """
            mutation($in: [SensorEventMutationInputArg!]!) {
                create_SensorEvent(input: $in) { _docID }
            }
        """,
        "variables": {"in": [event]}
    }


def main():
    print(f"[ingest] Serial={SERIAL_PORT}@{BAUD} GraphQL={GQL_URL}")
    try:
        ser = serial.Serial(SERIAL_PORT, BAUD, timeout=1)
    except Exception as e:
        print(f"Failed to open serial port {SERIAL_PORT}: {e}")
        sys.exit(1)

    while True:
        line = ser.readline().decode("utf-8", "ignore").strip()
        if not line:
            continue
        
        try:
            evt = json.loads(line)
        except json.JSONDecodeError:
            continue

        # Minimal validation (match Arduino JSON)
        if not all(k in evt for k in ("deviceId", "timestamp", "prediction", "probability")):
            continue


        body = dict(evt)
        print(body)

        try:
            r = requests.post(GQL_URL, json=gql_create(body), timeout=5)
            # Print the raw response for visibility while testing
            print("[resp]", r.status_code, r.text)

            if r.ok:
                data = r.json()
                if "errors" in data:
                    # Ignore duplicate docID error (content-addressed collision)
                    msg = json.dumps(data["errors"])
                    if "already exists" in msg:
                        print("[dup] identical document already stored, skipping.")
                    else:
                        print("[err]", msg[:200])
                else:
                    print(f"[ok] {evt['deviceId']} ts={evt['timestamp']} "
                          f"pred={evt['prediction']} p={float(evt['probability']):.3f}")
            else:
                print("[http-err]", r.status_code, r.text[:200])

        except Exception as e:
            print("[net-err]", e)
            time.sleep(1)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(0)
