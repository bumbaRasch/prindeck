# i18n glossary

Device terminology for the locale catalogs in `ui/src/locales/`. The English
catalog is the semantic source; each section below fixes the wording its locale
must use consistently. Sources: `docs/escl-l2710dw.md`, `docs/oids-l2710dw.md`,
`docs/ews-l2710dw.md`, PRD §3 and §4 (device facts and vocabulary), plus
standard desktop-UI conventions for each language.

Register: terse UI text, no exclamation style, sentence case where the language
uses it. Placeholders (`{count}`, `{pages}` …) are copied exactly from `en`.

## Plural categories and digits

| Locale | Plural forms | Digit convention |
|---|---|---|
| en | one, other | Western Arabic digits |
| ru | one, few, many, other | Western Arabic digits |
| uk | one, few, many, other | Western Arabic digits |
| de | one, other | Western Arabic digits |
| fr | one, many, other (`one` covers 0 and 1; `many` fires at exact millions) | Western Arabic digits |
| es | one, many (`many` fires at exact millions), other | Western Arabic digits |
| pt-BR | one, many (`many` fires at exact millions), other | Western Arabic digits |
| zh-CN | no plural (single form) | Western Arabic digits |
| ja | no plural (single form) | Western Arabic digits |
| ar | zero, one, two, few, many, other | Arabic-Indic digits for standalone numbers |

## English (source of truth)

| Term | Wording | Notes |
|---|---|---|
| scan | Scan / scanning | verb on buttons, noun elsewhere |
| platen | Flatbed | scan source |
| feeder / ADF | Feeder | scan source |
| duplex, long edge | Two-sided, long edge | print sides |
| duplex, short edge | Two-sided, short edge | print sides |
| toner | Toner | |
| drum | Drum | drum unit |
| cartridge model | Cartridge model | setting |
| cartridge yield | Cartridge yield | in pages |
| queue | Print queue | CUPS queue |
| jam(s) | jam(s) | paper jam counter |
| pages | page(s) / p. | full or abbreviated after a number |
| sides | Sides | print setting |
| offline / online | Offline / Online | status chip |
| web panel | Web panel | device web interface |
| firmware | Firmware | |

## Russian

Author-verified in `ru.ts`; the reference for register in every other language.

| Термин | Перевод | Примечания |
|---|---|---|
| scan | сканировать / сканирование | |
| platen | планшет | |
| feeder / ADF | автоподатчик | |
| duplex, long edge | по длинному краю | двусторонняя печать |
| duplex, short edge | по короткому краю | |
| toner | тонер | |
| drum | барабан | |
| cartridge model | модель картриджа | |
| cartridge yield | ресурс картриджа | в страницах |
| queue | очередь печати | |
| jam(s) | замятие / замятий | |
| pages | страница / стр. | полное или кратное после числа |
| sides | стороны | |
| offline / online | нет связи / на связи | |
| web panel | веб-панель | |
| firmware | прошивка | |

## Ukrainian

| Термін | Переклад | Примітки |
|---|---|---|
| scan | сканувати / сканування | |
| platen | планшет | |
| feeder / ADF | автоподатчик | |
| duplex, long edge | за довгим краєм | двосторонній друк |
| duplex, short edge | за коротким краєм | |
| toner | тонер | |
| drum | барабан | |
| cartridge model | модель картриджа | |
| cartridge yield | ресурс картриджа | у сторінках |
| queue | черга друку | |
| jam(s) | заминання / заминань | |
| pages | сторінка / стор. | повне або скорочене після числа |
| sides | сторони | |
| offline / online | немає зв'язку / на зв'язку | |
| web panel | веб-панель | |
| firmware | прошивка | |

## German

| Begriff | Wortlaut | Hinweise |
|---|---|---|
| scan | Scannen / Scan | |
| platen | Vorlageglas | Brother-DE-Terminus |
| feeder / ADF | Vorlagenzuführung | |
| duplex, long edge | Beidseitig, lange Kante | |
| duplex, short edge | Beidseitig, kurze Kante | |
| toner | Toner | |
| drum | Trommel | Trommeleinheit |
| cartridge model | Kartuschenmodell | |
| cartridge yield | Kartuschenkapazität | in Seiten |
| queue | Druckwarteschlange | |
| jam(s) | Papierstau(s) | |
| pages | Seite(n) | |
| sides | Seiten | Einseitig/beidseitig |
| offline / online | Offline / Online | |
| web panel | Webinterface | |
| firmware | Firmware | |

## French

| Terme | Libellé | Notes |
|---|---|---|
| scan | Numériser / numérisation | |
| platen | Vitre | vitre du scanner |
| feeder / ADF | Chargeur | chargeur automatique de documents |
| duplex, long edge | Recto verso, bord long | |
| duplex, short edge | Recto verso, bord court | |
| toner | Toner | |
| drum | Tambour | |
| cartridge model | Modèle de cartouche | |
| cartridge yield | Rendement de la cartouche | en pages |
| queue | File d'impression | |
| jam(s) | bourrage(s) | bourrage papier |
| pages | page(s) | |
| sides | Faces | recto seul / recto verso |
| offline / online | Hors ligne / En ligne | |
| web panel | Interface web | |
| firmware | Micrologiciel | |

## Spanish

| Término | Texto | Notas |
|---|---|---|
| scan | Escanear / escaneo | |
| platen | Cristal | cristal del escáner |
| feeder / ADF | Alimentador | |
| duplex, long edge | Dos caras, borde largo | |
| duplex, short edge | Dos caras, borde corto | |
| toner | Tóner | |
| drum | Tambor | |
| cartridge model | Modelo de cartucho | |
| cartridge yield | Rendimiento del cartucho | en páginas |
| queue | Cola de impresión | |
| jam(s) | atasco(s) | atasco de papel |
| pages | página(s) | |
| sides | Caras | |
| offline / online | Sin conexión / En línea | |
| web panel | Panel web | |
| firmware | Firmware | |

## Portuguese (Brazil)

| Termo | Texto | Notas |
|---|---|---|
| scan | Digitalizar / digitalização | |
| platen | Vidro | vidro do scanner |
| feeder / ADF | Alimentador | |
| duplex, long edge | Dois lados, borda longa | |
| duplex, short edge | Dois lados, borda curta | |
| toner | Toner | |
| drum | Cilindro | unidade de cilindro |
| cartridge model | Modelo do cartucho | |
| cartridge yield | Rendimento do cartucho | em páginas |
| queue | Fila de impressão | |
| jam(s) | atolamento(s) | atolamento de papel |
| pages | página(s) | |
| sides | Lados | |
| offline / online | Sem conexão / Conectado | |
| web panel | Painel web | |
| firmware | Firmware | |

## Chinese (Simplified)

| 术语 | 用词 | 说明 |
|---|---|---|
| scan | 扫描 | |
| platen | 平板 | |
| feeder / ADF | 输稿器 | 自动输稿器 |
| duplex, long edge | 双面，长边 | 长边装订 |
| duplex, short edge | 双面，短边 | 短边装订 |
| toner | 墨粉 | |
| drum | 硒鼓 | 鼓单元（DR 系列） |
| cartridge model | 墨粉盒型号 | TN 系列 |
| cartridge yield | 墨粉盒印量 | 以页计 |
| queue | 打印队列 | |
| jam(s) | 卡纸 | |
| pages | 页 | |
| sides | 单双面 | |
| offline / online | 离线 / 在线 | |
| web panel | 网页面板 | |
| firmware | 固件 | |

## Japanese

| 用語 | 表記 | 備考 |
|---|---|---|
| scan | スキャン | |
| platen | 原稿台 | |
| feeder / ADF | フィーダー | 自動原稿送り装置（ADF） |
| duplex, long edge | 両面（長辺とじ） | |
| duplex, short edge | 両面（短辺とじ） | |
| toner | トナー | |
| drum | ドラム | ドラムユニット |
| cartridge model | カートリッジ型番 | |
| cartridge yield | カートリッジ印字枚数 | ページ単位 |
| queue | 印刷キュー | |
| jam(s) | 紙づまり | |
| pages | ページ | |
| sides | 印刷面 | 片面/両面 |
| offline / online | オフライン / オンライン | |
| web panel | Web管理画面 | |
| firmware | ファームウェア | |

## Arabic

| المصطلح | الصياغة | ملاحظات |
|---|---|---|
| scan | مسح | |
| platen | الزجاج | سطح المسح |
| feeder / ADF | وحدة التغذية | وحدة التغذية التلقائية للمستندات |
| duplex, long edge | وجهان، الحافة الطويلة | |
| duplex, short edge | وجهان، الحافة القصيرة | |
| toner | التونر | |
| drum | الأسطوانة | وحدة الأسطوانة |
| cartridge model | طراز الخرطوشة | |
| cartridge yield | سعة الخرطوشة | بالصفحات |
| queue | قائمة الطباعة | |
| jam(s) | انحشار / انحشارات | انحشار الورق |
| pages | صفحة / صفحات | |
| sides | الوجوه | |
| offline / online | غير متصل / متصل | |
| web panel | لوحة الويب | |
| firmware | البرنامج الثابت | |
