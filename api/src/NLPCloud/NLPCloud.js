const NLPCloudClient = require('nlpcloud')

exports.makeClient = (token) => () => new NLPCloudClient('gpt-j', token,  gpu = false)

exports.generationImpl = (client) => ({ minLength, maxLength }) => (input) => () =>
    client.generation(input, minLength = minLength, maxLength = maxLength)
