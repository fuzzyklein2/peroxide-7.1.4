import os
from pathlib import Path

LEADING_SPACE = ' '
FOLDER_PICT = f"📁{LEADING_SPACE}"

def cwd():
    """Return the current working directory."""
    return Path.cwd()

def pwd():
    """Print `cwd()` and return it."""
    CWD = cwd()
    print(f'{FOLDER_PICT}Current working directory: {CWD}')
    return CWD

def cd(p:str|Path)->Path|None:
    """Change the current working directory."""
    p = Path(p)
    if not p.exists():
        print(f'{WARNING_PICT}WARNING: Directory {str(p)} does not exist!')
        return
    os.chdir(p)
    return p

def find_proj_root(output : bool = True # Whether to output the current directory after moving there.
                  ):
    """ Move to the project root directory. """
    if cwd().stem == 'lab': cd('../')
    # if output: pwd()

find_proj_root()
pwd()
