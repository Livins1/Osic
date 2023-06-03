/* @refresh reload */
// import './index.css'
// import { hydrate } from 'solid-js/web'
// import App from './App'

// UseFor SSR
// hydrate(() => <App />, document.getElementById('root') as HTMLElement)

/* @refresh reload */
import { render } from 'solid-js/web';
import { Router } from '@solidjs/router';

import './index.css';
import App from './App';

render(() =>
    <Router>
        <App />
    </Router>
    , document.getElementById('root') as HTMLElement);