const path = require('path');

config.client = config.client || {};
config.client.mocha = { ...config.client.mocha, timeout: 10000 };
config.files.push({
    pattern: path.join(config.basePath, 'composeResources/**/*'),
    watched: false,
    included: false,
    served: true,
});
