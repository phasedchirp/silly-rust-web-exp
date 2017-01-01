<html lang="en">
    <head>
        <meta charset="utf8"/>
    </head>
    <body>
        <h1>Users will see the following survey questions:</h1>
        <form method="post" action="/survey/created">
          {{#questions}}
            {{text}}<br><input type="text" name="q{{number}}"></br>
          {{/questions}}
          <input type="hidden" value="{{id}}" name="id" />
          <input type="radio" name="verify" value="good"> All correct<br>
          <input type="radio" name="verify" value="notgood"> Changes needed<br>
          <button type="submit">Confirm</button><br>
        </form>
    </body>
</html>
