# Проект по предмету "Рефакторинг баз данных и приложений"

Авторы:
 - Алексей Никашкин, P34101
 - Анна Марченко, P34101

Проект заключается в разработке браузерной онлайн-рисовалки с хранением и обработкой изображений на сервере.

## Сценарии использования

 1. Регистрация, авторизация пользователей (логин, пароль)
 2. Создание, удаление, просмотр списка своих холстов
 3. Preview изображений в списке холстов
 4. Базовое рисование на холсте (одна кисть, один размер кисти, один цвет)
 5. Просмотр истории изменений холста
 6. Изменение размера холста (с сохранением старого размера в истории)
 7. Продвинутое рисование на холсте (изменение размера кисти, выбор произвольного цвета)
 8. Серверная обработка изображений (размытие по гауссу, инвертирование цветов)

## Компоненты

 1. Фронтенд на React
   
    Персистентное хранилище данных - localStorage. Хранит токен авторизации, закешированный список проектов, preview, и т.д.
 
 2. Основной backend сервис на Rust.
 
    Имеет свою БД PostgreSQL для хранения аккаунтов пользователей, метаданных о их проектах, истории изменений не включая данные самих изображений.
 
    Реализует REST API для всего функционала, необходимого на фронтенде.

    Обращается к сервису хранения и обработки изображений.
 
 3. Сервис хранения и обработки изображений на Rust.
 
    Управляет файлами изображений в директории. Изображения статически раздаются по HTTP.
 
    Изображения не привязаны к конкретным пользователям (сервис ничего не знает о пользователях) и адресуются хешами. Изображения иммутабельны.
 
    Реализует REST API для создания новых изображений, внесения изменений в изображение, обработки изображений. При любых изменениях создается новое изображение, а старое остается прежним. Ответственность за удалением старых версий лежит на главном сервисе.
