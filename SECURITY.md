# Xavfsizlik

bitg kiritish usuli sifatida siz yozayotgan matnni ko'radi, shuning uchun xavfsizlik muammolarini jiddiy qabul qilamiz.

## Qo'llab-quvvatlanadigan versiyalar

| Versiya | Holat |
|---|---|
| 0.3.x | ✅ tuzatishlar chiqadi |
| < 0.3 | ❌ yangilang |

Xavfsizlik tuzatishlari faqat eng so'nggi minor versiyaga (hozir 0.3.x) chiqadi. Yangi minor versiya (masalan 0.4.0)
chiqqan kuni oldingisi qo'llab-quvvatlanmaydigan bo'ladi: tuzatish uchun yangi versiyaga o'ting. 1.0 dan oldin
eski versiyalarga alohida tuzatish (backport) chiqarilmaydi.

## Zaiflik haqida xabar berish

Ommaviy issue **ochmang**. GitHub'ning yopiq kanalidan foydalaning:
[**Report a vulnerability**](https://github.com/dtwinuz/bitg/security/advisories/new).

Xabarda: ta'sirlangan versiya va platforma, qayta chiqarish qadamlari, mumkin bo'lgan ta'sir.

## Xabar kelgandan keyin (Vulnerability response)

| Qadam | Muddat |
|---|---|
| Xabar olinganini tasdiqlash | 7 kun ichida |
| Tekshirish va xavf darajasi (CVSS) | 14 kun ichida |
| Tuzatish va reliz (yuqori/kritik) | 60 kun ichida, iloji bo'lsa tezroq |
| GitHub Security Advisory va CVE (kerak bo'lsa) | tuzatilgan reliz bilan birga |

Xabar beruvchi tuzatish ustida yopiq advisory'da ishtirok etadi. Tuzatish chiqqach, xohlasangiz, ismingiz
advisory'da va `CHANGELOG.md` da minnatdorchilik bilan qayd etiladi. Xavfsizlik asoslamasi va ma'lum kamchiliklar:
[docs/ASSURANCE.md](docs/ASSURANCE.md).

## bitg nimani qilmaydi

- Yozilgan matnni tarmoqqa yubormaydi va saqlamaydi (faqat siz qo'shgan istisno so'zlar `exceptions.txt` ga yoziladi).
- Linux'da parol, PIN va maxfiy maydonlarda o'chiq turadi; macOS'da tizim xavfsiz kiritish paytida kiritish
  usullarini o'chiradi; Windows'da terminallarda, shuningdek maydon turi parol, PIN yoki email bo'lsa
  (`InputScope`) yoki dastur klaviatura kiritishini o'chirgan bo'lsa o'chiq turadi.
- Kiritish usuli (Windows TSF, Linux IBus/Fcitx5) klaviatura hook'i va clipboard ishlatmaydi. Clipboard faqat
  siz `Ctrl+Shift+Y` (belgilangan matnni o'girish) ni bosganda vaqtincha ishlatiladi va asl holiga qaytariladi.

## Yuklab olingan faylni tekshirish

v0.3.1 dan boshlab har bir reliz fayli (MSI, `.deb`, `.pkg`) uchun GitHub Actions imzolagan SLSA build provenance va
SBOM (SPDX) attestation'i bor. Fayl aynan shu repodagi teg'dan, GitHub Actions'da yig'ilganini tekshirish:

```bash
gh attestation verify bitg-0.3.1-x64.msi -R dtwinuz/bitg
```

Har relizda `bitg-<versiya>.spdx.json` ham bor: unda bitg ichidagi barcha kutubxonalar va ularning versiyalari.

v0.3.2 dan boshlab paketlarni alohida reusable workflow (`build.yml`) yig'adi va imzolaydi (SLSA Build L3). Faylni
aynan shu workflow, aynan shu teg uchun yig'ganini qat'iy tekshirish:

```bash
gh attestation verify bitg-0.3.2-x64.msi -R dtwinuz/bitg \
  --signer-workflow dtwinuz/bitg/.github/workflows/build.yml --source-ref refs/tags/v0.3.2 --deny-self-hosted-runners
```

v0.3.1 uchun `--signer-workflow` qiymati `dtwinuz/bitg/.github/workflows/release.yml`.

Internetsiz (GitHub API'siz) tekshirish uchun relizdagi bundle fayli:

```bash
gh attestation verify bitg-0.3.1-x64.msi -R dtwinuz/bitg --bundle bitg-0.3.1.intoto.jsonl
```

bitg.uz dagi fayllar ham aynan shu reliz fayllari: deploy skripti ularni GitHub Release'dan oladi va attestation
tekshiruvidan o'tmasa to'xtaydi.

**Imzo haqida:** Windows MSI kod-imzo sertifikati bilan imzolanmagan (SmartScreen "noma'lum nashriyotchi" deydi),
macOS `.pkg` faqat ad-hoc imzolangan (Developer ID emas, notarizatsiya yo'q). Shuning uchun faylning haqiqiyligini
yuqoridagi `gh attestation verify` va `.sha256` bilan tekshiring.

bitg kodi muntazam fuzz qilinadi (`fuzz/`, libFuzzer): ixtiyoriy matnda yadro panic bermasligi va IME tahriri
so'zdan tashqariga chiqmasligi tekshiriladi.
