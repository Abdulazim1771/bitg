---
slug: /klaviatura/windows/
lang: uz
title: "Windows uchun yangi alifbo klaviaturasi (bepul) · bitg"
description: "Windows 10/11 da yangi o'zbek alifbosi harflarini yozish: bitg o'rnatish, tugmalar tartibi (Tilmoch, Turkcha Q, AltGr) va avtomatik o'girish. Bepul."
h1: "Windows'da yangi alifbo klaviaturasi"
schema: [SoftwareApplication, Article, BreadcrumbList]
updated: 2026-11-12
---

# Windows'da yangi alifbo klaviaturasi

**Qisqa javob:** bitg'ni o'rnating, **Win + Probel** bilan «bitg»ni tanlang va odatdagidek yozing. So'z tugashi
bilan (probel) u yangi alifboga o'tadi: `o'zbek` → `özbek`, `choy` → `çoy`. Xohlasangiz, yangi harflar to'g'ridan-
to'g'ri tugmada ham bo'ladi. Bepul, ochiq kod, internetsiz.

[Windows uchun yuklab olish](/yuklab-olish/)

## O'rnatish — 3 qadam
1. `bitg-<versiya>-x64.msi` faylini oching (Windows 10/11, x64). O'rnatuvchi barcha foydalanuvchilar uchun
   ishlaydi va klaviaturani O'zbek (lotin) tiliga o'zi qo'shadi.
2. **Win + Probel** → «bitg».
3. Yozing. Kompyuterni qayta yuklash yoki ochiq dasturlarni yopish shart emas.

> SmartScreen «Windows kompyuteringizni himoya qildi» desa: **Batafsil → Baribir ishga tushirish**.
> Faylning SHA-256 yig'indisi va manba kodi [yuklab olish sahifasida](/yuklab-olish/).

## Ikki rejim

| Rejim | Nima qiladi | Kimga |
|---|---|---|
| **Avtomatik o'girish** | eskicha yozilgan so'z probeldan keyin yangi alifboga o'tadi | eski odat bilan yozadiganlar |
| **Yangi harflar tugmada** | tugma darhol ö ğ ş ç yozadi | yangi alifboda to'g'ridan-to'g'ri yozmoqchilar |

Ikkalasi alohida yoqiladi va birga ishlashi mumkin.

## Tugmalar tartiblari

| Tartib | ö | ğ | ş | ç | ʼ | Izoh |
|---|---|---|---|---|---|---|
| **Tilmoch** (standart) | `[` | `]` | `;` | `'` | `` ` `` | qavslar va `;` `'` — AltGr bilan |
| Turkcha Q | `,` | `[` | `;` | `.` | — | vergul `\`, nuqta `/` da; `i` → ı |
| Faqat AltGr | AltGr+O | AltGr+G | AltGr+S | AltGr+C | — | hech narsa ko'chmaydi |
| Kirill | — | — | — | — | — | Windows «O'zbek (kirill)»; avto o'girish yoqiq bo'lsa so'z yangi lotinga o'tadi |
| Maxsus | istalgan | | | | | 4 qatlam: oddiy, Shift, AltGr, AltGr+Shift |

## Inglizcha va kod buzilmaydimi?

Yo'q. bitg so'z o'zbekcha ekaniga dalil bo'lmasa, unga tegmaydi: `check`, `show`, `Obama's` o'zgarmaydi;
`Menga chat yozing` → `Menga çat yozing`. Havola, email, `@user`, `#teg`, `snake_case`, `camelCase` va
parolga o'xshash so'zlar ham o'zgarmaydi. O'lchov: inglizcha gaplarning 99.73% i o'zgarmay qoladi, o'zbekcha
gaplarda aniqlik 99.17%.

## Xato o'girdi — nima qilaman?

So'z o'girilgandan keyin darhol **Backspace** bosing — asl so'z qaytadi va shu dasturda boshqa o'girilmaydi.
Doimiy istisno ro'yxati sozlamalarda (qidirish va o'chirish bilan).

## Qaysi dasturlarda ishlaydi

| Dastur turi | Holat |
|---|---|
| Word, Chrome, Edge, WPF/UWP dasturlari | ✅ |
| Telegram Desktop va eski dasturlar | ✅ zaxira yo'l orqali |
| Terminallar (Windows Terminal, PuTTY...) | standart holatda o'chiq (buyruq va parol yoziladi), yoqish mumkin |

Har dastur uchun alohida yoqish/o'chirish sozlamalarda bor.

## bitg'siz yo'l
Windows sozlamalariga «Turkish Q» tartibini qo'shish mumkin — lekin unda vergul, nuqta va `i` harfi joyi
o'zgaradi, eski matnlarni esa o'girmaydi. Batafsil: [Ş, Ç, Ğ, Ö qanday yoziladi](/yangi-harflarni-yozish/).

## Kompaniyada
Jim o'rnatish: `msiexec /i bitg-<versiya>-x64.msi /qn`. GPO, Intune, SCCM — [tashkilotlar uchun](/tashkilotlar/).

*Yangilangan: 2026-11-12*

**Bog'liq:** [Yuklab olish](/yuklab-olish/) · [Klaviatura tartiblari](/klaviatura/tartiblar/) ·
[Yangi harflarni yozish](/yangi-harflarni-yozish/) · [Tashkilotlar](/tashkilotlar/)
