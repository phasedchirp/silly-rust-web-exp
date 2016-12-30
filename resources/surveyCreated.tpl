<html lang="en">
    <head>
        <meta charset="utf8"/>
    </head>
    <body>
        <h1>You entered the following questions:</h1>
        <form method="post" action="/makeSurvey">
          {{#questions}}
            {{text}}<br><input type="text" name="q{{number}}"></br>
          {{/questions}}
          <button type="submit">Looks correct?</button><br/>
        </form>
    </body>
</html>
