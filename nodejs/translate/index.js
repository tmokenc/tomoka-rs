'use strict'

const trans = require('google-translate-api')

function parseConfig(text) {
  const lang = {
    from: 'auto',
    to:   'vi'
  }

  const ex = text.match(/\[(.*)?\]/)
  if (ex) {
    let v = ex[1].split('>')

    if (v[0])          lang.from = v[0]
    if (v[0] === 'vi') lang.to = 'en'
    if (v[1])          lang.to = v[1]

    text = text.replace(ex[0], '').trim()
  }

  return { text, lang }
}

function tieqViet(tiengViet) {
  const key = {
    'c(?!(h|H))|q': 'k',     'C(?!(h|H))|Q': 'K',
    'c(h|H)|t(r|R)': 'c',    'C(h|H)|T(r|R)': 'C',
    'd|g(i|I)|r': 'z',       'D|G(i|I)|R': 'Z',
    'g(i|í|ì|ĩ|ỉ|ị)': 'z$1', 'G(i|í|ì|ĩ|ỉ|ị)': 'Z$1',
    'Đ': 'D',                'đ': 'd',
    'G(h|H)': 'G',           'g(h|H)': 'g',
    'p(h|H)': 'f',           'P(h|H)': 'F',
    'n((g|G)(h|H)?)': 'q',   'N((g|G)(h|H)?)': 'Q',
    'k(h|H)': 'x',           'K(h|H)': 'X',
    't(h|H)': 'w',           'T(h|H)': 'W',
    'n(h|H)': 'n\'',         'N(h|H)': 'N\''
  }

  for (const k in key) {
    tiengViet = tiengViet.replace(new RegExp(k, 'g'), key[k]);
  }

  return tiengViet
}

(async function(content) {
    const { text, lang } = parseConfig(content)
    
    if (!text) {
        console.error("Please enter something to translate")
        return
    }
    
    let translation
    try {
        translation = await trans(text, lang)
    } catch (err) {
        console.error("Cannot get the translation from google translate")
        return
    }
    
    const { iso, didYouMean } = translation.from.language
    const {
      value,
      autoCorrected: textAutoCorrect,
      didYouMean: textDidYouMean
    } = translation.from.text
    
    const ntv = iso !== 'vi' || lang.to !== 'vi'
    
    const language = {
        text: ntv ? translation.text : tieqViet(text),
        from: trans.languages[iso],
        to: ntv 
            ? trans.languages[lang.to]
              ? trans.languages[lang.to]
              : lang.to
            : 'Tiếq Việt',
        didYouMean,
    }
    
    if (textAutoCorrect || textDidYouMean) {
        language.autoCorrected = value.replace(/\[|\]/g, '___')
    }
    
    console.log(JSON.stringify(language))

    // const embed = {
    //   title: `${language.from} -> ${language.to}`,
    //   description: '=> ' + language.text
    // }
// 
    // if (didYouMean) {
    //   embed.fields = [{
    //     name: 'Did you mean',
    //     value: 'language: ' + trans.languages[iso],
    //     inline: true
    //   }]
    // }
// 
    // if (textAutoCorrect || textDidYouMean) {
    //   const i = {
    //     name: 'Autocorrected',
    //     value: value.replace(/\[|\]/g, '___'),
    //     inline: true
    //   }
// 
    //   embed.fields ? embed.fields.push(i) : embed.fields = [i]
    // }
})(process.argv.slice(2).join(" "))