import json, time

from watchdog import WatchDog
from database import Fire

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

def get_db():
    data = {}
    for fire in Fire.select():
        fire: Fire
        contours = {}
        for c in fire.contours:
            if c.thresh <= 10:
                contours["low"] = json.loads(c.data)
            elif c.thresh<= 180:
                contours["med"] = json.loads(c.data)
            else:
                contours["high"] = json.loads(c.data)
            

        data[fire.title] = {
            "last_updated": fire.updated,
            "contours": contours,
            "lat" : fire.lat,
            "long": fire.long,
            "temp": fire.temp,
            "under_control": fire.under_control
        }
        
    return data

app = FastAPI()
app.mount("/static", StaticFiles(directory="static"), name="static")
templates = Jinja2Templates(directory="templates")

origins = ["*"]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/latest_data.json")
async def state():
    return {"last_updated":time.time(),"data":get_db()}

@app.get("/")
async def index(request: Request):
    return templates.TemplateResponse("index.html", {"request": request})

@app.get("/panic.html")
async def index(request: Request):
    return templates.TemplateResponse("panic.html", {"request": request})

if __name__ == "__main__":
    import uvicorn
    WatchDog().start()
    uvicorn.run(app, host="0.0.0.0", port=8000)