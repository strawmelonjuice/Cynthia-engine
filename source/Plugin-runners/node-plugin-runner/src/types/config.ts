export default interface Config {
	port: number;
	cache: {
		max_cache_size: number;
		lifetimes: {
			stylesheets: number;
			javascript: number;
			forwarded: number;
			assets: number;
			served: number;
		};
	};
	site: {
		notfound_page: string;
		site_baseurl: string;
		og_sitename: string;
		meta: {
			enable_tags: boolean;
		};
	};
	logs: {
		file_loglevel: number;
		term_loglevel: number;
		logfile: string;
	};
	runtimes: {
		ext_js_rt: string;
	};
	scenes: Array<{
		name: string;
		sitename: string;
		stylefile: string;
		script: string;
		templates: {
			post: string;
			page: string;
			postlist: string;
		};
	}>;
	plugins: Array<{
		plugin_name: string;
		plugin_enabled: boolean;
		plugin_runtime: string;
	}>;
}
