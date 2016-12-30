<html lang="en">
    <head>
        <meta charset="utf8"/>
    </head>
    <body>
        <h1>You entered the following questions:</h1>
        <form method="post" action="/makeSurvey">
          {{#questions}}
            <br><input type="text" name="q{{number}}">{{text}}</br>
          {{/questions}}
          <button type="submit">Create Survey</button><br/>
        </form>
        <p>Hopefully they're correct</p>
    </body>
</html>
