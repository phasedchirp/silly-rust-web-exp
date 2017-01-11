<html>
    <body>
        <h1>
            Your survey has been successfully created
        </h1>
          <p>Your survey ID is: {{id}}<br>
          Your survey key is: {{key}}<br>
          You will need these to access results and remove the survey when finished</p>

          <p>Your survey can be accessed at {{ path }}</p>
          <p>To view results, you can go to {{ results}} where &ltformat&gt is the format you'd like the output in (currently only .csv is supported).</p>
          <p>To remove the survey, go to {{ delete}} and your survey will be removed.</p>
    </body>
</html>
