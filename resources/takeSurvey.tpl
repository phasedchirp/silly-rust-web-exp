<html lang="en">
    <head>
        <meta charset="utf8"/>
    </head>
    <body>
        <h1>Please complete the following questions:</h1>
        <form method="post" action="/submitSurvey">
          {{#questions}}
            {{text}}<br><input type="text" name="q{{number}}"></br>
          {{/questions}}
          <button type="submit">Submit</button><br>
        </form>
    </body>
</html>
