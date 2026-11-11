//! Zaxira yo'l uchun: dastur matnni TSF'ga bermasa (Telegram Desktop), bosilgan harflardan joriy so'zni yig'amiz.
//! Faqat so'z chegarasidan (probel, Enter, fokus) keyin ketma-ket yozilgan harflar ishonchli — navigatsiya,
//! Ctrl+..., noma'lum belgi bo'lsa so'z "noaniq" bo'ladi va o'girilmaydi (begona matnni o'chirib yubormaslik uchun).

/// Bitta so'z uchun eng ko'p belgi: bundan uzun — noaniq.
const MAX: usize = 40;

#[derive(Debug, Default)]
pub struct Typed {
    word: Vec<char>,
    /// So'z boshi ko'rilgan (chegaradan keyin boshlangan) va oraliqda shubhali hodisa bo'lmagan.
    trusted: bool,
}

impl Typed {
    /// Fokus o'zgardi yoki so'z tugadi: keyingi harf — yangi ishonchli so'z boshi.
    pub fn boundary(&mut self) {
        self.word.clear();
        self.trusted = true;
    }

    /// Navigatsiya, sichqoncha, Ctrl+V, noma'lum belgi: kursor qayerdaligini endi bilmaymiz.
    pub fn lost(&mut self) {
        self.word.clear();
        self.trusted = false;
    }

    pub fn push(&mut self, c: char) {
        if self.word.len() >= MAX {
            self.trusted = false;
        }
        self.word.push(c);
    }

    pub fn backspace(&mut self) {
        if self.word.pop().is_none() {
            // so'zdan oldingi matn o'chirilmoqda — keyingi so'z boshi noma'lum
            self.trusted = false;
        }
    }

    /// Probel bosildi: ishonchli so'z bo'lsa — uni qaytaradi. Har holda chegara qo'yiladi.
    pub fn finish(&mut self) -> Option<String> {
        let w = (self.trusted && !self.word.is_empty() && self.word.len() <= MAX)
            .then(|| self.word.iter().collect());
        self.boundary();
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(s: &str) -> Typed {
        let mut t = Typed::default();
        t.boundary();
        s.chars().for_each(|c| t.push(c));
        t
    }

    #[test]
    fn collects_word_after_boundary() {
        let mut t = typed("choy");
        assert_eq!(t.finish().as_deref(), Some("choy"));
        "shahar".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish().as_deref(), Some("shahar"));
    }

    #[test]
    fn backspace_edits_word() {
        let mut t = typed("chox");
        t.backspace();
        t.push('y');
        assert_eq!(t.finish().as_deref(), Some("choy"));
    }

    #[test]
    fn untrusted_after_navigation_or_deleting_past_start() {
        let mut t = typed("ch");
        t.lost();
        "oy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish(), None);

        let mut t = typed("a");
        t.backspace();
        t.backspace(); // oldingi so'zga kirdi
        "choy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish(), None);
        // keyingi so'z yana ishonchli
        "choy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish().as_deref(), Some("choy"));
    }

    #[test]
    fn nothing_before_first_boundary() {
        let mut t = Typed::default();
        "choy".chars().for_each(|c| t.push(c));
        assert_eq!(t.finish(), None);
    }
}
