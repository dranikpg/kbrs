### Лабораторная работа N1

Алгоритмы реализованы на ЯП Rust.

В качестве исходных данных я выбрал датасет из 3 тыс. статей https://www.kaggle.com/datasets/asad1m9a9h6mood/news-articles.

| Длина ключевого слова      | Процент успеха | Минимальная длина взломанного текста|
| ----------- | ----------- | ----------- |
| 1      | 43%       |  1765 |
| 2      | 36%       |  1765 |
| 3      | 27%       |  1765 |
| 5      | 14%       |  1775 |
| 10      | 3%       |  2119 |
| 25     | 0.3%       |  3549 |
| 50     | 0.03%       |  7463 |


```
Sucess rate for word len 1: 1183 / 2692 = 0.43945023, min text size decrypted ... 1756
Sucess rate for word len 2: 971 / 2692 = 0.36069837, min text size decrypted ... 1756
Sucess rate for word len 3: 740 / 2692 = 0.27488855, min text size decrypted ... 1759
Sucess rate for word len 5: 391 / 2692 = 0.14524516, min text size decrypted ... 1775
Sucess rate for word len 10: 99 / 2692 = 0.03677563, min text size decrypted ... 2119
Sucess rate for word len 25: 10 / 2692 = 0.0037147102, min text size decrypted ... 3549
Sucess rate for word len 50: 1 / 2692 = 0.00037147102, min text size decrypted ... 7463
```
