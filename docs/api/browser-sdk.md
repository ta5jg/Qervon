# Qervon Browser SDK

`/static/qervon-client.js` provides a cookie-session client for customer,
courier and operations endpoints. It automatically sends the CSRF header for
write operations and never stores bearer credentials in browser storage.

```js
import { QervonClient } from '/static/qervon-client.js';
const qervon = new QervonClient();
const orders = await qervon.customerOrders();
```

The SDK deliberately exposes only authenticated tenant-scoped APIs.
