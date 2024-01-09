import json
import base64


def main():
    data = ""

    with open("./out.json", "r") as file:
        data = json.loads(file.read())

    fdata = data["file"]
    ext = data["metadata"]["file_ext"]

    decoded_file = base64.b64decode(str(fdata))

    print(len(decoded_file))

    with open(f"img.{ext}", "wb") as file:
        file.write(decoded_file)


if __name__ == "__main__":
    main()
