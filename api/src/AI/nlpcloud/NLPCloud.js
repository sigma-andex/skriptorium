const NLPCloudClient = require('nlpcloud')

exports.makeClientImpl = (token) => () => new NLPCloudClient('gpt-j', token, gpu = false)

exports.generationImpl = (client) => ({ minLength,
    maxLength,
    lengthNoInput,
    endSequence,
    removeInput,
    doSample,
    numBeams,
    earlyStopping,
    noRepeatNgramSize,
    numReturnSequences,
    topK,
    topP,
    temperature,
    repetitionPenalty,
    lengthPenalty,
    badWords }) => (input) => () =>
        client.generation(input, minLength, maxLength,
            lengthNoInput,
            endSequence,
            removeInput,
            doSample,
            numBeams,
            earlyStopping,
            noRepeatNgramSize,
            numReturnSequences,
            topK,
            topP,
            temperature,
            repetitionPenalty,
            lengthPenalty,
            badWords)
