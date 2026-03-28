
impl BlkScaner {

    // another thread
    fn do_serve(&self, wkr: Worker) {
        // ctx
        let cnf = self.cnf.clone();
        let dbconn = self.dbconn.clone();
        let setting = self.setting.clone();
        // let diamovedate = self.diamovedate.clone();
        // server listen loop with multi thread
        let rt = node::new_tokio_rt( true );
        rt.block_on(async move {
            crate::server::server_listen(cnf, setting, dbconn, wkr).await;
        });
    }

}
