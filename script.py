import os
from pathlib import Path

home = Path.home()
desktop = next((p for p in [home/"Desktop", home/"Escritorio", home/"OneDrive"/"Desktop"] if p.exists()), home)
base = desktop / "Simulacion_Windows"

carpetas = [
    "Windows/System32", "Windows/System32/drivers", "Windows/System32/drivers/etc",
    "Windows/SysWOW64", "Windows/Fonts", "Windows/Temp", "Windows/Logs",
    "Windows/Prefetch", "Windows/WinSxS",
    "Program Files", "Program Files (x86)", "Program Files/Common Files",
    "Program Files/Internet Explorer", "Program Files/Windows Defender",
    "Program Files/Windows NT/accessories",
    "Users/Public", "Users/Public/Documents", "Users/Public/Downloads",
    "Users/Usuario/Desktop", "Users/Usuario/Documents", "Users/Usuario/Downloads",
    "Users/Usuario/AppData/Local/Temp", "Users/Usuario/AppData/Roaming",
    "ProgramData", "ProgramData/Package Cache",
    "PerfLogs/Admin", "Temp",
]

archivos = [
    "bootmgr", "BOOTNXT", "pagefile.sys", "hiberfil.sys", "swapfile.sys",
    "Windows/explorer.exe", "Windows/notepad.exe", "Windows/regedit.exe",
    "Windows/win.ini", "Windows/system.ini", "Windows/Temp/~DF1A2B.tmp",
    "Windows/Logs/CBS.log", "Windows/Prefetch/EXPLORER.EXE-1234.pf",
    "Windows/System32/cmd.exe", "Windows/System32/calc.exe",
    "Windows/System32/mspaint.exe", "Windows/System32/taskmgr.exe",
    "Windows/System32/kernel32.dll", "Windows/System32/user32.dll",
    "Windows/System32/ntdll.dll", "Windows/System32/advapi32.dll",
    "Windows/System32/drivers/etc/hosts",
    "Windows/System32/drivers/null.sys",
    "Windows/System32/drivers/tcpip.sys",
    "Program Files/Internet Explorer/iexplore.exe",
    "Program Files/Windows Defender/MsMpEng.exe",
    "Program Files/Windows NT/accessories/wordpad.exe",
    "Program Files (x86)/Common Files/ole32.dll",
    "Users/Public/Documents/documento.txt",
    "Users/Public/Downloads/setup.exe",
    "Users/Usuario/Desktop/acceso directo.lnk",
    "Users/Usuario/Documents/notas.txt",
    "Users/Usuario/Downloads/instalador.msi",
    "Users/Usuario/AppData/Local/Temp/temp1.tmp",
    "Users/Usuario/AppData/Roaming/config.ini",
    "Users/Usuario/NTUSER.DAT",
    "ProgramData/Package Cache/cache.dat",
    "PerfLogs/Admin/report.xml", "Temp/archivo.tmp",
]

# Crear carpetas
for c in carpetas:
    (base / c).mkdir(parents=True, exist_ok=True)

# Crear archivos con contenido
n = 0
for a in archivos:
    ruta = base / a
    ruta.parent.mkdir(parents=True, exist_ok=True)
    if not ruta.exists():
        contenido = (
            "=== SECRET CONTENT ===\n"
            f"Archivo: {ruta.name}\n"
            f"Ruta:    {a}\n"
            "Estado:  dummy file (simulacion)\n"
            "==============================\n"
            "Este archivo no contiene datos reales.\n"
            "Solo se usa para simular la estructura de Windows.\n"
        )
        ruta.write_text(contenido, encoding="utf-8")
        n += 1

print(f"✔ {len(carpetas)} carpetas y {n} archivos creados en:\n  {base}")

if os.name == "nt":
    os.startfile(base)