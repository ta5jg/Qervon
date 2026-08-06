#!/usr/bin/env python3
"""Qervon projesindeki tüm .md dosyalarını PDF kaynaklarına göre güncelleyen script.

Bu script:
1. `docs/sources/qervon-1.pdf` ve `docs/sources/qervon-2.pdf` dosyalarından metin çıkartır.
2. Çıkarılan metni özetleyerek (ilk 500 karakter) bir açıklama oluşturur.
3. Projenin kök dizinindeki tüm `.md` dosyalarını bulur.
4. Her `.md` dosyasının başına standart bir giriş ekler.
5. Ayrıca `docs/qervon-1.md` ve `docs/qervon-2.md` adlı dosyalar oluşturur ve tam PDF metnini bu dosyalara yazar.

Kullanım:
    python3 scripts/update_docs.py
"""
import os
from pathlib import Path
# pyrefly: ignore [missing-import]
from PyPDF2 import PdfReader

ROOT = Path(__file__).resolve().parents[1]  # proje kök dizini
PDF_DIR = ROOT / "docs" / "sources"
PDF1_PATH = PDF_DIR / "qervon-1.pdf"
PDF2_PATH = PDF_DIR / "qervon-2.pdf"

def extract_text(pdf_path: Path) -> str:
    reader = PdfReader(str(pdf_path))
    text = []
    for page in reader.pages:
        try:
            text.append(page.extract_text() or "")
        except Exception:
            continue
    return "\n".join(text)

def summarize(text: str, limit: int = 500) -> str:
    # basit özet: ilk limit karakter
    return text[:limit].replace("\n", " ") + ("..." if len(text) > limit else "")

def main():
    pdf1_text = extract_text(PDF1_PATH)
    pdf2_text = extract_text(PDF2_PATH)
    pdf1_summary = summarize(pdf1_text)
    pdf2_summary = summarize(pdf2_text)

    header_template = f"""---
Bu dosya Qervon projesi dokümantasyonu, `qervon-1.pdf` ve `qervon-2.pdf` içeriklerine dayalı olarak güncellenmiştir.

**PDF 1 özeti:**
{pdf1_summary}

**PDF 2 özeti:**
{pdf2_summary}

*Bu kısım otomatik olarak oluşturulmuştur.*
---

"""


    # Tüm .md dosyalarını bul ve güncelle
    for md_path in ROOT.rglob("*.md"):
        # .md dosyasının içeriğini okuma
        try:
            original = md_path.read_text(encoding="utf-8")
        except Exception as e:
            print(f"[WARN] {md_path} okunamadı: {e}")
            continue
        # zaten bizim header var mı kontrol
        if "Qervon projesi dokümantasyonu" in original:
            continue  # tekrar eklemeyi önle
        new_content = header_template + original
        md_path.write_text(new_content, encoding="utf-8")
        print(f"[INFO] {md_path} güncellendi.")

    # PDF metinlerini ayrı .md dosyalarına yaz
    out1 = ROOT / "docs" / "qervon-1.md"
    out2 = ROOT / "docs" / "qervon-2.md"
    out1.write_text(pdf1_text, encoding="utf-8")
    out2.write_text(pdf2_text, encoding="utf-8")
    print("[INFO] PDF tam metinleri qervon-1.md ve qervon-2.md olarak oluşturuldu.")

if __name__ == "__main__":
    main()
