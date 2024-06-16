import json
import requests
from bs4 import BeautifulSoup

myOpsCodesfile = open("./ops_codes.json")
myOpsCodes = json.loads(myOpsCodesfile.read())
myOpsCodesfile.close()

resp = requests.get("http://www.obelisk.me.uk/6502/reference.html")
resp.raise_for_status()

soup = BeautifulSoup(resp.text, 'html.parser')

opsCodes = {}

for table in soup.find_all('table'):
    for row in table.find_all('tr'):
        cols = row.find_all('td')
        cols = [ele.text.strip() for ele in cols]
        if len(cols) != 4 or 'Home' in cols or 'Cycles' in cols:
            continue
        opsCodes[cols[1]] = {
            'opcode': cols[1],
            'cycles': int(cols[-1].split(' ')[0]),
            '+1': True if len(cols[-1].split(' ')) > 1 else False
        }

for item in myOpsCodes:
    meta = opsCodes.get(item['opcode'], {'cycles': 0, '+1': False})
    item.update(meta)

illegal_op_codes = [
    {
    
    }
]

myOpsCodesfile = open("./ops_codes.json", 'w')
myOpsCodesfile.write(json.dumps(myOpsCodes))
