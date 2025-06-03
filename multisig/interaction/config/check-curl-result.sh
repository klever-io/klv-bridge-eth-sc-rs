check_curl_error() {
    if [ $? -ne 0 ];
    then
        printf "Curl error\n\n"
        exit 1
    fi
    # jq for error 
    ERROR=$(jq -r '.error' <<< "${QUERY_RESULT}")
    # check if error is not empty
    if [ ! -z "${ERROR}" ];    
    then
        printf "Error: ${ERROR}\n\n"
        exit 1
    fi
}

check_result() {
    # check if SC_RESULT is empty
    if [ -z "${SC_RESULT}" ];
    then
        printf "SC_RESULT is empty\n\n"
        exit 1
    fi
    # jq SC_RESULT for error
    STATUS=$(jq -r '.status' <<< "${SC_RESULT}")
    printf "Status: ${STATUS}\n\n"

    # check if status is success
    if [ ${STATUS} != "success" ];
    then
        printf "Error: ${SC_RESULT}\n\n"
        exit 1
    fi
}