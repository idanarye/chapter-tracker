(defproject chapter-tracker "0.2.0-SNAPSHOT"
  :description "FIXME: write description"
  :url "http://example.com/FIXME"
  :main chapter-tracker.core
  :license {:name "Eclipse Public License"
            :url "http://www.eclipse.org/legal/epl-v10.html"}
  :dependencies [
                 [org.clojure/clojure "1.5.1"]
                 [org.clojure/java.jdbc "0.0.6"]
                 [org.xerial/sqlite-jdbc "3.7.2"]
                 ;[seesaw "1.4.2"] ;I ditched seesaw because it makes the compilation painfully slow
                ])
