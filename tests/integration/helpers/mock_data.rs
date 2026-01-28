pub(crate) fn get_mock_table_rows_response() -> serde_json::Value {
    serde_json::json!({
      "meta": {
        "containerName": "timeregistration",
        "containerInstanceId": "e9dcec2c-ba66-4b4b-a970-162455905253"
      },
      "panes": {
        "card": {
          "meta": {
            "paneName": "card",
            "rowCount": 1,
            "rowOffset": 0
          },
          "records": [
            {
              "data": {
                "employeenumber": "12345",
                "periodstartvar": "2024-10-21",
                "periodendvar": "2024-10-27",
                "employeenamevar": "John Smith",
                "datevar": "2024-10-24",
                "weeknumbervar": 43,
                "partvar": "",
                "fixednumberday1var": 8,
                "fixednumberday2var": 0,
                "fixednumberday3var": 0,
                "fixednumberday4var": 8,
                "fixednumberday5var": 8,
                "fixednumberday6var": 0,
                "fixednumberday7var": 0,
                "totalnumberday1var": 8,
                "totalnumberday2var": 8,
                "totalnumberday3var": 8,
                "totalnumberday4var": 0,
                "totalnumberday5var": 0,
                "totalnumberday6var": 0,
                "totalnumberday7var": 0,
                "regulartimeday1var": 8,
                "regulartimeday2var": 8,
                "regulartimeday3var": 8,
                "regulartimeday4var": 0,
                "regulartimeday5var": 0,
                "regulartimeday6var": 0,
                "regulartimeday7var": 0
              }
            }
          ]
        },
        "table": {
          "meta": {
            "paneName": "table",
            "rowCount": 3,
            "rowOffset": 0
          },
          "records": [
            {
              "data": {
                "jobnumber": "ABC123",
                "numberday1": 8,
                "numberday2": 0,
                "numberday3": 0,
                "numberday4": 0,
                "numberday5": 0,
                "numberday6": 0,
                "numberday7": 0,
                "entrytext": "Some task one",
                "taskname": "300",
                "approvalstatus": "",
                "instancekey": "1579ecb8-7773-4b69-b3ff-116da9dee8d8",
                "timeregistrationunit": "hours",
                "jobnamevar": "Job One",
                "tasktextvar": "Some task one"
              }
            },
            {
              "data": {
                "jobnumber": "DEF456",
                "numberday1": 0,
                "numberday2": 0,
                "numberday3": 0,
                "numberday4": 0,
                "numberday5": 0,
                "numberday6": 0,
                "numberday7": 0,
                "entrytext": "Job Two",
                "taskname": "Some task two",
                "approvalstatus": "",
                "instancekey": "265123e0-a069-44d2-bd60-8706f1a7d9b9",
                "timeregistrationunit": "hours",
                "jobnamevar": "Job One",
                "tasktextvar": "Some task two"
              }
            }
          ]
        }
      }
    })
}
