from main import get_db
import time, os, json, hashlib, asyncio, gists, shutil
from git import Repo
from watchdog import WatchDog

LOCAL_PATH = 'tmp'
JSON_FILE = 'latest_data.json' 
REPO_URL = 'https://github.com/Aveygo/NuclearSmokeData.git'

dog = WatchDog()
dog.start()

def sha256sum(filename):
    with open(filename, 'rb', buffering=0) as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def pull_repo():
    if os.path.exists(LOCAL_PATH):
        shutil.rmtree(LOCAL_PATH)
    repo = Repo.clone_from(REPO_URL, LOCAL_PATH)
    return repo

def update_json_file():
    json_file_path = os.path.join(LOCAL_PATH, JSON_FILE)
    data = {"last_updated":time.time(),"data":get_db()}
    with open(json_file_path, 'w') as f:
        json.dump(data, f, indent=4)

def commit_and_squash(repo):
    repo.git.add(all=True)

    repo.git.commit('-m', 'Update data')
    repo.git.reset('--soft', 'HEAD~1')
    repo.git.commit('--amend', '-m', 'Squash commits')
    repo.git.push('origin', 'main', force=True)

while True:
    
    while dog.working:
        time.sleep(1)

    if dog.last_checked > time.time() - 60 * 30:

        repo = pull_repo()
        update_json_file()
        commit_and_squash(repo)
                    
    else:
        print("CRITICAL! Watchdog likely dead / overwhelmed!")

    time.sleep(60 * 60)
    
    

    
